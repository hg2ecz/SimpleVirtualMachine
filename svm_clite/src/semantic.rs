use crate::ast::*;
use std::collections::HashMap;

pub fn validate(p: &Program) -> Result<(), String> {
    let mut funcs: HashMap<&str, &Function> = HashMap::new();
    for f in &p.functions {
        if matches!(f.name.as_str(), "true" | "false") {
            return Err(format!("{} is a reserved boolean literal", f.name));
        }
        if funcs.insert(&f.name, f).is_some() {
            return Err(format!("duplicate function {}", f.name));
        }
    }
    if !funcs.contains_key("main") {
        return Err("program must define main()".into());
    }

    let mut globals = HashMap::<String, Ty>::new();
    for g in &p.globals {
        if matches!(g.name.as_str(), "true" | "false") {
            return Err(format!("{} is a reserved boolean literal", g.name));
        }
        if g.ty == Ty::Void {
            return Err(format!("global {} cannot have type void", g.name));
        }
        if let Ty::Array(elem, n) = &g.ty {
            if *n == 0 {
                return Err(format!("array {} must have at least one element", g.name));
            }
            if matches!(&**elem, Ty::Void | Ty::Array(_, _) | Ty::Ptr(_)) {
                return Err(format!(
                    "array {} must contain bool/i8/u8/i16/u16 elements",
                    g.name
                ));
            }
            let bytes = u32::from(elem.width()) * u32::from(*n);
            if bytes > u32::from(u16::MAX) {
                return Err(format!(
                    "array {} occupies {bytes} bytes, beyond the 16-bit address space",
                    g.name
                ));
            }
            if g.init.is_some() {
                return Err(format!("array initializer for {} is not supported", g.name));
            }
        }
        if globals.insert(g.name.clone(), g.ty.clone()).is_some() {
            return Err(format!("duplicate global {}", g.name));
        }
    }
    for g in &p.globals {
        if funcs.contains_key(g.name.as_str()) {
            return Err(format!(
                "global {} conflicts with a function of the same name",
                g.name
            ));
        }
        if let Some(e) = &g.init {
            if !matches!(e, Expr::Int(_) | Expr::Bool(_)) {
                return Err(format!(
                    "global initializer for {} must be a scalar literal in C-Lite",
                    g.name
                ));
            }
            let et = expr_ty(e, &globals, &funcs)?;
            require_assignable(&g.ty, &et, &format!("initializer of global {}", g.name))?;
        }
    }
    for f in &p.functions {
        validate_fn(f, &funcs, &globals)?;
    }
    validate_no_recursion(p)?;
    Ok(())
}

fn validate_no_recursion(p: &Program) -> Result<(), String> {
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for f in &p.functions {
        let mut calls = Vec::new();
        collect_calls_block(&f.body, &mut calls);
        graph.insert(f.name.clone(), calls);
    }
    fn visit(
        n: &str,
        graph: &HashMap<String, Vec<String>>,
        visiting: &mut Vec<String>,
        done: &mut std::collections::HashSet<String>,
    ) -> Result<(), String> {
        if done.contains(n) {
            return Ok(());
        }
        if let Some(pos) = visiting.iter().position(|x| x == n) {
            let mut cycle = visiting[pos..].to_vec();
            cycle.push(n.to_string());
            return Err(format!(
                "recursion/cyclic calls are not supported: {}",
                cycle.join(" -> ")
            ));
        }
        visiting.push(n.to_string());
        if let Some(next) = graph.get(n) {
            for m in next {
                if graph.contains_key(m) {
                    visit(m, graph, visiting, done)?;
                }
            }
        }
        visiting.pop();
        done.insert(n.to_string());
        Ok(())
    }
    let mut done = std::collections::HashSet::new();
    for f in &p.functions {
        visit(&f.name, &graph, &mut Vec::new(), &mut done)?;
    }
    Ok(())
}

fn collect_calls_block(body: &[Stmt], out: &mut Vec<String>) {
    for s in body {
        match s {
            Stmt::Let { init, .. } => {
                if let Some(e) = init {
                    collect_calls_expr(e, out);
                }
            }
            Stmt::Assign { lhs, rhs } => {
                collect_calls_expr(lhs, out);
                collect_calls_expr(rhs, out);
            }
            Stmt::Expr(e) | Stmt::Return(Some(e)) => collect_calls_expr(e, out),
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                collect_calls_expr(cond, out);
                collect_calls_block(then_body, out);
                collect_calls_block(else_body, out);
            }
            Stmt::While { cond, body } => {
                collect_calls_expr(cond, out);
                collect_calls_block(body, out);
            }
            Stmt::Return(None) | Stmt::Break | Stmt::Continue => {}
        }
    }
}
fn collect_calls_expr(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Call { name, args } => {
            out.push(name.clone());
            for a in args {
                collect_calls_expr(a, out);
            }
        }
        Expr::Unary { expr, .. } => collect_calls_expr(expr, out),
        Expr::Binary { lhs, rhs, .. } => {
            collect_calls_expr(lhs, out);
            collect_calls_expr(rhs, out);
        }
        Expr::Index { base, index } => {
            collect_calls_expr(base, out);
            collect_calls_expr(index, out);
        }
        Expr::Bool(_) | Expr::Int(_) | Expr::Var(_) => {}
    }
}

fn validate_fn(
    f: &Function,
    funcs: &HashMap<&str, &Function>,
    globals: &HashMap<String, Ty>,
) -> Result<(), String> {
    if matches!(f.ret, Ty::Array(_, _)) {
        return Err(format!("function {} cannot return an array", f.name));
    }
    let mut env = globals.clone();
    for p in &f.params {
        if matches!(p.name.as_str(), "true" | "false") {
            return Err(format!("{} is a reserved boolean literal", p.name));
        }
        if matches!(p.ty, Ty::Array(_, _)) {
            return Err(format!(
                "array parameter {}.{} is not allowed; use *T",
                f.name, p.name
            ));
        }
        if p.ty == Ty::Void {
            return Err(format!(
                "parameter {}.{} cannot have type void",
                f.name, p.name
            ));
        }
        if f.params.iter().filter(|q| q.name == p.name).count() > 1 {
            return Err(format!("duplicate parameter {}.{}", f.name, p.name));
        }
        if globals.contains_key(&p.name) {
            return Err(format!(
                "parameter {}.{} shadows global {}; C-Lite has one simple variable namespace",
                f.name, p.name, p.name
            ));
        }
        env.insert(p.name.clone(), p.ty.clone());
    }
    let mut local_names = env
        .keys()
        .cloned()
        .collect::<std::collections::HashSet<_>>();
    validate_block(&f.body, f, funcs, &mut env, &mut local_names, 0)
}

fn validate_block(
    body: &[Stmt],
    f: &Function,
    funcs: &HashMap<&str, &Function>,
    env: &mut HashMap<String, Ty>,
    local_names: &mut std::collections::HashSet<String>,
    loop_depth: usize,
) -> Result<(), String> {
    for s in body {
        match s {
            Stmt::Let { name, ty, init } => {
                if matches!(name.as_str(), "true" | "false") {
                    return Err(format!("{} is a reserved boolean literal", name));
                }
                if *ty == Ty::Void {
                    return Err(format!("local {name} cannot have type void"));
                }
                if let Ty::Array(elem, n) = ty {
                    if *n == 0 {
                        return Err(format!("array {name} must have at least one element"));
                    }
                    if matches!(&**elem, Ty::Void | Ty::Array(_, _)) {
                        return Err(format!("array {name} must contain scalar elements"));
                    }
                    let bytes = u32::from(elem.width()) * u32::from(*n);
                    if bytes > u32::from(u16::MAX) {
                        return Err(format!(
                            "array {name} occupies {bytes} bytes, beyond the 16-bit address space"
                        ));
                    }
                    if init.is_some() {
                        return Err(format!("array initializer for {name} is not supported"));
                    }
                }
                if !local_names.insert(name.clone()) {
                    return Err(format!(
                        "local name {name} is already used in {}; C-Lite keeps one static slot per local name",
                        f.name
                    ));
                }
                if let Some(e) = init {
                    let et = expr_ty(e, env, funcs)?;
                    require_assignable(ty, &et, &format!("initializer of {name}"))?;
                }
                env.insert(name.clone(), ty.clone());
            }
            Stmt::Assign { lhs, rhs } => {
                let lt = lvalue_ty(lhs, env, funcs)?;
                let rt = expr_ty(rhs, env, funcs)?;
                require_assignable(&lt, &rt, "assignment")?;
            }
            Stmt::Expr(e) => {
                expr_ty(e, env, funcs)?;
            }
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                require_condition(&expr_ty(cond, env, funcs)?)?;
                let mut a = env.clone();
                validate_block(then_body, f, funcs, &mut a, local_names, loop_depth)?;
                let mut b = env.clone();
                validate_block(else_body, f, funcs, &mut b, local_names, loop_depth)?;
            }
            Stmt::While { cond, body } => {
                require_condition(&expr_ty(cond, env, funcs)?)?;
                let mut inner = env.clone();
                validate_block(body, f, funcs, &mut inner, local_names, loop_depth + 1)?;
            }
            Stmt::Return(e) => match (&f.ret, e) {
                (Ty::Void, None) => {}
                (Ty::Void, Some(_)) => {
                    return Err(format!("void function {} cannot return a value", f.name));
                }
                (_, None) => return Err(format!("function {} must return a value", f.name)),
                (rt, Some(e)) => {
                    let et = expr_ty(e, env, funcs)?;
                    require_assignable(rt, &et, &format!("return from {}", f.name))?;
                }
            },
            Stmt::Break | Stmt::Continue if loop_depth == 0 => {
                return Err("break/continue is only valid inside a loop".into());
            }
            Stmt::Break | Stmt::Continue => {}
        }
    }
    Ok(())
}

fn scalar(t: &Ty) -> bool {
    matches!(
        t,
        Ty::Bool | Ty::I8 | Ty::U8 | Ty::I16 | Ty::U16 | Ty::Ptr(_)
    )
}

fn require_condition(t: &Ty) -> Result<(), String> {
    if scalar(t) {
        Ok(())
    } else {
        Err("condition must be a scalar value".into())
    }
}

fn require_assignable(dst: &Ty, src: &Ty, where_: &str) -> Result<(), String> {
    if dst == src {
        return Ok(());
    }
    // Integer literals are represented as U16 in the tiny AST. Permit them to
    // flow into scalar integer destinations; range checks are handled by the
    // backend semantics for now. Other mixed-width assignments remain explicit.
    if *src == Ty::U16 && matches!(dst, Ty::I8 | Ty::U8 | Ty::I16 | Ty::U16) {
        return Ok(());
    }
    Err(format!(
        "type mismatch in {where_}: expected {}, got {}",
        dst.name(),
        src.name()
    ))
}

fn lvalue_ty(
    e: &Expr,
    env: &HashMap<String, Ty>,
    funcs: &HashMap<&str, &Function>,
) -> Result<Ty, String> {
    match e {
        Expr::Var(n) => {
            let t = env
                .get(n)
                .cloned()
                .ok_or_else(|| format!("unknown variable {n}"))?;
            if matches!(t, Ty::Array(_, _)) {
                Err(format!("array {n} is not directly assignable"))
            } else {
                Ok(t)
            }
        }
        Expr::Index { .. }
        | Expr::Unary {
            op: UnaryOp::Deref, ..
        } => expr_ty(e, env, funcs),
        _ => Err("left side of assignment is not assignable".into()),
    }
}

pub fn expr_ty(
    e: &Expr,
    env: &HashMap<String, Ty>,
    funcs: &HashMap<&str, &Function>,
) -> Result<Ty, String> {
    match e {
        Expr::Bool(_) => Ok(Ty::Bool),
        Expr::Int(_) => Ok(Ty::U16),
        Expr::Var(n) => env
            .get(n)
            .cloned()
            .ok_or_else(|| format!("unknown variable {n}")),
        Expr::Unary {
            op: UnaryOp::AddrOf,
            expr,
        } => match &**expr {
            Expr::Var(n) => {
                let t = env
                    .get(n)
                    .cloned()
                    .ok_or_else(|| format!("unknown variable {n}"))?;
                match t {
                    Ty::Array(elem, _) => Ok(Ty::Ptr(elem)),
                    other => Ok(Ty::Ptr(Box::new(other))),
                }
            }
            Expr::Index { base, .. } => {
                let elem = indexed_elem_ty(base, env, funcs)?;
                Ok(Ty::Ptr(Box::new(elem)))
            }
            _ => Err("address-of requires variable or indexed element".into()),
        },
        Expr::Unary {
            op: UnaryOp::Deref,
            expr,
        } => match expr_ty(expr, env, funcs)? {
            Ty::Ptr(t) => Ok(*t),
            _ => Err("dereference requires pointer".into()),
        },
        Expr::Unary { expr, .. } => {
            let t = expr_ty(expr, env, funcs)?;
            if scalar(&t) && !matches!(t, Ty::Ptr(_)) {
                Ok(t)
            } else {
                Err("numeric unary operator requires integer".into())
            }
        }
        Expr::Index { base, index } => {
            let it = expr_ty(index, env, funcs)?;
            if !matches!(it, Ty::I8 | Ty::U8 | Ty::I16 | Ty::U16) {
                return Err("array index must be integer".into());
            }
            let bt = expr_ty(base, env, funcs)?;
            if let (Ty::Array(_, n), Expr::Int(i)) = (&bt, &**index) {
                if *i >= *n {
                    return Err(format!(
                        "constant array index {} is out of bounds for length {}",
                        i, n
                    ));
                }
            }
            match bt {
                Ty::Array(t, _) | Ty::Ptr(t) => Ok(*t),
                _ => Err("indexing requires array or pointer".into()),
            }
        }
        Expr::Call { name, args } => {
            if let Some((ret, params)) = memory_builtin(name) {
                if args.len() != params.len() {
                    return Err(format!(
                        "{} expects {} arguments, got {}",
                        name,
                        params.len(),
                        args.len()
                    ));
                }
                for (i, (a, p)) in args.iter().zip(params.iter()).enumerate() {
                    let at = expr_ty(a, env, funcs)?;
                    require_assignable(p, &at, &format!("argument {} of {}", i + 1, name))?;
                }
                return Ok(ret);
            }
            let f = funcs
                .get(name.as_str())
                .ok_or_else(|| format!("unknown function {name}"))?;
            if args.len() != f.params.len() {
                return Err(format!(
                    "{} expects {} arguments, got {}",
                    name,
                    f.params.len(),
                    args.len()
                ));
            }
            for (i, (a, p)) in args.iter().zip(&f.params).enumerate() {
                let at = expr_ty(a, env, funcs)?;
                require_assignable(&p.ty, &at, &format!("argument {} of {}", i + 1, name))?;
            }
            Ok(f.ret.clone())
        }
        Expr::Binary { op, lhs, rhs } => {
            if matches!(op, BinaryOp::Div | BinaryOp::Mod) && matches!(&**rhs, Expr::Int(0)) {
                return Err("division/modulo by constant zero".into());
            }
            let lt = expr_ty(lhs, env, funcs)?;
            let rt = expr_ty(rhs, env, funcs)?;
            match op {
                BinaryOp::Eq | BinaryOp::Ne => {
                    if !scalar(&lt) || !scalar(&rt) {
                        return Err("comparison requires scalar operands".into());
                    }
                    Ok(Ty::Bool)
                }
                BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                    if matches!(&lt, Ty::Bool)
                        || matches!(&rt, Ty::Bool)
                        || !scalar(&lt)
                        || !scalar(&rt)
                    {
                        return Err(
                            "ordered comparison requires integer or pointer operands".into()
                        );
                    }
                    Ok(Ty::Bool)
                }
                _ => {
                    if !matches!(lt, Ty::I8 | Ty::U8 | Ty::I16 | Ty::U16)
                        || !matches!(rt, Ty::I8 | Ty::U8 | Ty::I16 | Ty::U16)
                    {
                        return Err("arithmetic/bit operator requires integer operands".into());
                    }
                    Ok(lt)
                }
            }
        }
    }
}

fn memory_builtin(name: &str) -> Option<(Ty, Vec<Ty>)> {
    Some(match name {
        "load8" | "vload8" => (Ty::U8, vec![Ty::U16]),
        "load16" | "vload16" => (Ty::U16, vec![Ty::U16]),
        "store8" | "vstore8" => (Ty::Void, vec![Ty::U16, Ty::U8]),
        "store16" | "vstore16" => (Ty::Void, vec![Ty::U16, Ty::U16]),
        _ => return None,
    })
}

fn indexed_elem_ty(
    base: &Expr,
    env: &HashMap<String, Ty>,
    funcs: &HashMap<&str, &Function>,
) -> Result<Ty, String> {
    match expr_ty(base, env, funcs)? {
        Ty::Array(t, _) | Ty::Ptr(t) => Ok(*t),
        _ => Err("indexing requires array or pointer".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn array_pointer_call_is_valid() {
        let p = parse("fn sum(u16* p,u16 n)->u16{return p[0];} fn main()->u16{u16 values[4]; values[0]=1; return sum(&values[0],4);}").unwrap();
        validate(&p).unwrap();
    }

    #[test]
    fn array_assignment_is_rejected() {
        let p = parse("fn main()->u16{u16 a[4]; u16 b[4]; a=b; return 0;}").unwrap();
        assert!(
            validate(&p)
                .unwrap_err()
                .contains("not directly assignable")
        );
    }

    #[test]
    fn wrong_pointer_element_type_is_rejected() {
        let p = parse("fn f(u16* p)->u16{return 0;} fn main()->u16{u8 a[4]; return f(&a[0]);}")
            .unwrap();
        assert!(validate(&p).unwrap_err().contains("type mismatch"));
    }

    #[test]
    fn constant_array_bounds_are_checked() {
        let p = parse("fn main()->u16{u16 a[4]; return a[4];}").unwrap();
        assert!(validate(&p).unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn globals_are_visible_in_functions() {
        let p = parse("u16 counter=3; u8 buf[8]; fn main()->u16{buf[0]=1; return counter+buf[0];}")
            .unwrap();
        validate(&p).unwrap();
    }
    #[test]
    fn direct_recursion_is_rejected() {
        let p = parse("fn f(u16 n)->u16{return f(n);} fn main()->u16{return f(1);}").unwrap();
        assert!(validate(&p).unwrap_err().contains("recursion/cyclic calls"));
    }

    #[test]
    fn mutual_recursion_is_rejected() {
        let p =
            parse("fn a()->u16{return b();} fn b()->u16{return a();} fn main()->u16{return a();}")
                .unwrap();
        assert!(validate(&p).unwrap_err().contains("recursion/cyclic calls"));
    }

    #[test]
    fn raw_memory_builtins_are_architecture_neutral() {
        let p = parse("fn main()->u16{vstore8(0xff00,65);return load16(0x1000);}").unwrap();
        validate(&p).unwrap();
    }

    #[test]
    fn raw_memory_builtin_checks_value_width() {
        let p = parse("fn f(u16 x){store8(0x1000,&x);} fn main()->u16{return 0;}").unwrap();
        assert!(validate(&p).unwrap_err().contains("argument 2 of store8"));
    }

    #[test]
    fn constant_division_by_zero_is_rejected() {
        let p = parse("fn main()->u16{return 7/0;}").unwrap();
        assert!(validate(&p).unwrap_err().contains("constant zero"));
    }

    #[test]
    fn local_names_are_unique_across_function_blocks() {
        let p = parse("fn main()->u16{if(1){u16 x=1;}else{u16 x=2;}return 0;}").unwrap();
        assert!(
            validate(&p)
                .unwrap_err()
                .contains("one static slot per local name")
        );
    }

    #[test]
    fn bool_is_one_byte_logical_type() {
        let p = parse("fn less(u16 a,u16 b)->bool{return a<b;} fn main()->u16{bool b=less(1,2); if(b){return 1;} return 0;}").unwrap();
        validate(&p).unwrap();
        assert_eq!(Ty::Bool.width(), 1);
    }

    #[test]
    fn integer_is_not_implicitly_assignable_to_bool() {
        let p = parse("fn main()->u16{bool b=1; return 0;}").unwrap();
        assert!(validate(&p).unwrap_err().contains("type mismatch"));
    }

    #[test]
    fn ordered_bool_comparison_is_rejected() {
        let p =
            parse("fn main()->u16{bool a=true;bool b=false;if(a<b){return 1;}return 0;}").unwrap();
        assert!(validate(&p).unwrap_err().contains("ordered comparison"));
    }
}
