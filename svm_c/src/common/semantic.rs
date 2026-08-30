use crate::model::*;
use std::collections::{HashMap, HashSet};

pub(crate) fn validate(p: &Program) -> Result<(), String> {
    let mut names = HashSet::new();
    for g in &p.globals {
        if !names.insert(g.name.clone()) {
            return Err(format!("duplicate global '{}'", g.name));
        }
        if g.len != 1 && g.init.is_some() {
            return Err(format!("array '{}' initializer is not supported", g.name));
        }
        if let Some(e) = &g.init {
            if g.ty.is_wide() {
                return Err(format!(
                    "wide global '{}' must be initialized through the arithmetic library",
                    g.name
                ));
            }
            if !matches!(e, Expr::Num(_)) {
                return Err(format!(
                    "global '{}' initializer must be a constant",
                    g.name
                ));
            }
        }
    }
    let mut funcs = HashSet::new();
    for f in &p.functions {
        if f.ret.is_wide() {
            return Err(format!(
                "function '{}' cannot return a wide type; pass a destination address instead",
                f.name
            ));
        }
        if !funcs.insert(f.name.clone()) {
            return Err(format!("duplicate function '{}'", f.name));
        }
        if f.params.len() > 4 {
            return Err(format!("function '{}' has more than 4 parameters", f.name));
        }
        let mut local = HashSet::new();
        for a in &f.params {
            if a.ty.is_wide() {
                return Err(format!(
                    "wide parameter '{}' in '{}' is not passed by value; use a u16 address parameter",
                    a.name, f.name
                ));
            }
            if a.ty == Ty::Void {
                return Err("void parameter is not allowed".into());
            }
            if !local.insert(a.name.clone()) {
                return Err(format!("duplicate parameter '{}'", a.name));
            }
        }
        collect_local_names(&f.body, &mut local)?;
    }
    if !funcs.contains("main") {
        return Err("program must define main()".into());
    }
    validate_semantics(p)?;
    let known: HashSet<String> = p.functions.iter().map(|f| f.name.clone()).collect();
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for f in &p.functions {
        let mut calls = Vec::new();
        calls_stmt(&f.body, &mut calls);
        calls.retain(|n| known.contains(n));
        graph.insert(f.name.clone(), calls);
    }
    let mut visiting = HashSet::new();
    let mut done = HashSet::new();
    for name in graph.keys() {
        reject_recursive(name, &graph, &mut visiting, &mut done)?;
    }
    Ok(())
}
pub(crate) fn validate_semantics(p: &Program) -> Result<(), String> {
    let globals: HashMap<String, Ty> = p.globals.iter().map(|g| (g.name.clone(), g.ty)).collect();
    let funcs: HashMap<String, FunctionSig> = p
        .functions
        .iter()
        .map(|f| {
            (
                f.name.clone(),
                FunctionSig {
                    ret: f.ret,
                    params: f.params.iter().map(|a| a.ty).collect(),
                },
            )
        })
        .collect();

    for name in globals.keys() {
        if funcs.contains_key(name) {
            return Err(format!(
                "name '{}' is used by both a global and a function",
                name
            ));
        }
    }

    for f in &p.functions {
        let mut vars = globals.clone();
        for a in &f.params {
            vars.insert(a.name.clone(), a.ty);
        }
        collect_var_types(&f.body, &mut vars);
        check_stmt_semantics(&f.body, f, &vars, &funcs)?;
    }
    Ok(())
}

fn collect_var_types(s: &Stmt, vars: &mut HashMap<String, Ty>) {
    match s {
        Stmt::Var(v) => {
            vars.insert(v.name.clone(), v.ty);
        }
        Stmt::Block(v) => {
            for x in v {
                collect_var_types(x, vars);
            }
        }
        Stmt::If(_, a, b) => {
            collect_var_types(a, vars);
            if let Some(b) = b {
                collect_var_types(b, vars);
            }
        }
        Stmt::While(_, b) | Stmt::DoWhile(b, _) => collect_var_types(b, vars),
        Stmt::For(a, _, st, b) => {
            if let Some(a) = a {
                collect_var_types(a, vars);
            }
            if let Some(st) = st {
                collect_var_types(st, vars);
            }
            collect_var_types(b, vars);
        }
        _ => {}
    }
}

pub(crate) fn builtin_sig(name: &str) -> Option<FunctionSig> {
    match name {
        "load8" => Some(FunctionSig {
            ret: Ty::U8,
            params: vec![Ty::U16],
        }),
        "load16" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![Ty::U16],
        }),
        "store8" => Some(FunctionSig {
            ret: Ty::Void,
            params: vec![Ty::U16, Ty::U8],
        }),
        "store16" => Some(FunctionSig {
            ret: Ty::Void,
            params: vec![Ty::U16, Ty::U16],
        }),
        "vload8" => Some(FunctionSig {
            ret: Ty::U8,
            params: vec![Ty::U16],
        }),
        "vload16" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![Ty::U16],
        }),
        "vstore8" => Some(FunctionSig {
            ret: Ty::Void,
            params: vec![Ty::U16, Ty::U8],
        }),
        "vstore16" => Some(FunctionSig {
            ret: Ty::Void,
            params: vec![Ty::U16, Ty::U16],
        }),
        "putc" => Some(FunctionSig {
            ret: Ty::Void,
            params: vec![Ty::U8],
        }),
        "puts" => Some(FunctionSig {
            ret: Ty::Void,
            params: vec![],
        }),
        "getc" => Some(FunctionSig {
            ret: Ty::U8,
            params: vec![],
        }),
        "clock_lo" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![],
        }),
        "clock_hi" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![],
        }),
        "instr_lo" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![],
        }),
        "instr_hi" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![],
        }),
        "asr1" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![Ty::U16],
        }),
        "mul_q15" => Some(FunctionSig {
            ret: Ty::U16,
            params: vec![Ty::U16, Ty::U16],
        }),
        _ => None,
    }
}

fn check_expr_semantics(
    e: &Expr,
    vars: &HashMap<String, Ty>,
    funcs: &HashMap<String, FunctionSig>,
    value_required: bool,
) -> Result<Ty, String> {
    let ty = match e {
        Expr::Num(_) => Ty::U16,
        Expr::Str(_) => return Err("string literal is only valid as the argument of puts()".into()),
        Expr::SizeOfName(_) => Ty::U16,
        Expr::AddrOf(n) => {
            if !vars.contains_key(n) {
                return Err(format!("unknown object '{}' in address-of", n));
            }
            Ty::U16
        }
        Expr::Var(n) => {
            let t = *vars
                .get(n)
                .ok_or_else(|| format!("unknown variable '{}'", n))?;
            if t.is_wide() && value_required {
                return Err(format!(
                    "wide object '{}' is address-only; use '&{}' with the integer/float library",
                    n, n
                ));
            }
            t
        }
        Expr::Unary(_, a) => {
            let t = check_expr_semantics(a, vars, funcs, true)?;
            if t == Ty::Void {
                return Err("void expression used as a value".into());
            }
            Ty::U16
        }
        Expr::Binary(_, a, b) => {
            let ta = check_expr_semantics(a, vars, funcs, true)?;
            let tb = check_expr_semantics(b, vars, funcs, true)?;
            if ta == Ty::Void || tb == Ty::Void {
                return Err("void expression used in binary operation".into());
            }
            Ty::U16
        }
        Expr::Call(n, args) => {
            if n == "puts" {
                if args.len() != 1 || !matches!(&args[0], Expr::Str(_)) {
                    return Err("puts() expects exactly one string literal".into());
                }
                return Ok(Ty::Void);
            }
            if let Some(name) = n.strip_prefix("__index__") {
                if args.len() != 1 {
                    return Err("internal array index arity error".into());
                }
                let ty = *vars
                    .get(name)
                    .ok_or_else(|| format!("unknown array '{}'", name))?;
                let _ = check_expr_semantics(&args[0], vars, funcs, true)?;
                return Ok(ty);
            }
            if let Some(name) = n.strip_prefix("__store__") {
                if args.len() != 2 {
                    return Err("internal array store arity error".into());
                }
                if !vars.contains_key(name) {
                    return Err(format!("unknown array '{}'", name));
                }
                let _ = check_expr_semantics(&args[0], vars, funcs, true)?;
                let _ = check_expr_semantics(&args[1], vars, funcs, true)?;
                return Ok(Ty::Void);
            }
            let sig = builtin_sig(n)
                .or_else(|| funcs.get(n).cloned())
                .ok_or_else(|| format!("unknown function '{}'", n))?;
            if args.len() != sig.params.len() {
                return Err(format!(
                    "function '{}' expects {} argument(s), got {}",
                    n,
                    sig.params.len(),
                    args.len()
                ));
            }
            for a in args {
                let _ = check_expr_semantics(a, vars, funcs, true)?;
            }
            sig.ret
        }
    };
    if value_required && ty == Ty::Void {
        return Err("void function call used as a value".into());
    }
    Ok(ty)
}

fn check_stmt_semantics(
    s: &Stmt,
    f: &Function,
    vars: &HashMap<String, Ty>,
    funcs: &HashMap<String, FunctionSig>,
) -> Result<(), String> {
    match s {
        Stmt::Block(v) => {
            for x in v {
                check_stmt_semantics(x, f, vars, funcs)?;
            }
        }
        Stmt::Var(v) => {
            if v.ty.is_wide() && v.init.is_some() {
                return Err(format!(
                    "wide object '{}' must be initialized through the arithmetic library",
                    v.name
                ));
            }
            if let Some(e) = &v.init {
                let _ = check_expr_semantics(e, vars, funcs, true)?;
            }
        }
        Stmt::Assign(n, e) => {
            if !vars.contains_key(n) {
                return Err(format!(
                    "assignment to unknown variable '{}' in function '{}'",
                    n, f.name
                ));
            }
            if vars.get(n).copied().map(Ty::is_wide).unwrap_or(false) {
                return Err(format!(
                    "wide object '{}' is address-only; use the integer/float library",
                    n
                ));
            }
            let _ = check_expr_semantics(e, vars, funcs, true)?;
        }
        Stmt::Expr(e) => {
            let _ = check_expr_semantics(e, vars, funcs, false)?;
        }
        Stmt::If(c, a, b) => {
            let _ = check_expr_semantics(c, vars, funcs, true)?;
            check_stmt_semantics(a, f, vars, funcs)?;
            if let Some(b) = b {
                check_stmt_semantics(b, f, vars, funcs)?;
            }
        }
        Stmt::While(c, b) => {
            let _ = check_expr_semantics(c, vars, funcs, true)?;
            check_stmt_semantics(b, f, vars, funcs)?;
        }
        Stmt::DoWhile(b, c) => {
            check_stmt_semantics(b, f, vars, funcs)?;
            let _ = check_expr_semantics(c, vars, funcs, true)?;
        }
        Stmt::Break | Stmt::Continue => {}
        Stmt::For(a, c, st, b) => {
            if let Some(a) = a {
                check_stmt_semantics(a, f, vars, funcs)?;
            }
            if let Some(c) = c {
                let _ = check_expr_semantics(c, vars, funcs, true)?;
            }
            if let Some(st) = st {
                check_stmt_semantics(st, f, vars, funcs)?;
            }
            check_stmt_semantics(b, f, vars, funcs)?;
        }
        Stmt::Return(e) => match (f.ret, e) {
            (Ty::Void, None) => {}
            (Ty::Void, Some(_)) => {
                return Err(format!("void function '{}' cannot return a value", f.name));
            }
            (_, None) => {
                return Err(format!(
                    "non-void function '{}' must return a value",
                    f.name
                ));
            }
            (_, Some(e)) => {
                let _ = check_expr_semantics(e, vars, funcs, true)?;
            }
        },
        Stmt::Empty => {}
    }
    Ok(())
}

fn calls_expr(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Call(n, a) => {
            if !n.starts_with("__index__")
                && !n.starts_with("__store__")
                && !matches!(
                    n.as_str(),
                    "load8"
                        | "load16"
                        | "store8"
                        | "store16"
                        | "vload8"
                        | "vload16"
                        | "vstore8"
                        | "vstore16"
                        | "putc"
                        | "puts"
                        | "getc"
                        | "clock_lo"
                        | "clock_hi"
                        | "instr_lo"
                        | "instr_hi"
                        | "asr1"
                        | "mul_q15"
                )
            {
                out.push(n.clone())
            }
            for x in a {
                calls_expr(x, out)
            }
        }
        Expr::Unary(_, a) => calls_expr(a, out),
        Expr::Binary(_, a, b) => {
            calls_expr(a, out);
            calls_expr(b, out)
        }
        Expr::Str(_) | Expr::SizeOfName(_) | Expr::AddrOf(_) => {}
        _ => {}
    }
}
fn calls_stmt(s: &Stmt, out: &mut Vec<String>) {
    match s {
        Stmt::Block(v) => {
            for x in v {
                calls_stmt(x, out)
            }
        }
        Stmt::Var(v) => {
            if let Some(e) = &v.init {
                calls_expr(e, out)
            }
        }
        Stmt::Assign(_, e) | Stmt::Expr(e) => calls_expr(e, out),
        Stmt::If(c, a, b) => {
            calls_expr(c, out);
            calls_stmt(a, out);
            if let Some(b) = b {
                calls_stmt(b, out)
            }
        }
        Stmt::While(c, b) => {
            calls_expr(c, out);
            calls_stmt(b, out)
        }
        Stmt::DoWhile(b, c) => {
            calls_stmt(b, out);
            calls_expr(c, out)
        }
        Stmt::Break | Stmt::Continue => {}
        Stmt::For(a, c, st, b) => {
            if let Some(a) = a {
                calls_stmt(a, out)
            }
            if let Some(c) = c {
                calls_expr(c, out)
            }
            if let Some(st) = st {
                calls_stmt(st, out)
            }
            calls_stmt(b, out)
        }
        Stmt::Return(e) => {
            if let Some(e) = e {
                calls_expr(e, out)
            }
        }
        Stmt::Empty => {}
    }
}
fn reject_recursive(
    name: &str,
    g: &HashMap<String, Vec<String>>,
    visiting: &mut HashSet<String>,
    done: &mut HashSet<String>,
) -> Result<(), String> {
    if done.contains(name) {
        return Ok(());
    }
    if !visiting.insert(name.to_string()) {
        return Err(format!(
            "recursive call cycle involving '{name}' is not supported by the static-local ABI"
        ));
    }
    if let Some(v) = g.get(name) {
        for n in v {
            reject_recursive(n, g, visiting, done)?
        }
    }
    visiting.remove(name);
    done.insert(name.to_string());
    Ok(())
}
fn collect_local_names(s: &Stmt, set: &mut HashSet<String>) -> Result<(), String> {
    match s {
        Stmt::Var(v) => {
            if !set.insert(v.name.clone()) {
                return Err(format!("duplicate local '{}'", v.name));
            }
        }
        Stmt::Block(v) => {
            for x in v {
                collect_local_names(x, set)?
            }
        }
        Stmt::If(_, a, b) => {
            collect_local_names(a, set)?;
            if let Some(b) = b {
                collect_local_names(b, set)?
            }
        }
        Stmt::While(_, b) | Stmt::DoWhile(b, _) => collect_local_names(b, set)?,
        Stmt::For(a, _, c, b) => {
            if let Some(a) = a {
                collect_local_names(a, set)?
            };
            if let Some(c) = c {
                collect_local_names(c, set)?
            };
            collect_local_names(b, set)?
        }
        _ => {}
    }
    Ok(())
}

/// True when an expression contains a function/builtin call.
///
/// This is a language-semantics property, not an optimization, because
/// compound array updates must not evaluate a side-effecting index twice.
pub(crate) fn expr_has_call(e: &Expr) -> bool {
    match e {
        Expr::Call(_, _) => true,
        Expr::Unary(_, a) => expr_has_call(a),
        Expr::Binary(_, a, b) => expr_has_call(a) || expr_has_call(b),
        _ => false,
    }
}
