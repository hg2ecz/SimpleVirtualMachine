use crate::model::*;
use crate::semantic::expr_has_call;
use std::collections::{HashMap, HashSet};

pub(crate) fn optimize_program(mut p: Program, level: OptLevel) -> Program {
    if level == OptLevel::O0 {
        return p;
    }
    for g in &mut p.globals {
        if let Some(e) = g.init.take() {
            g.init = Some(opt_expr(e));
        }
    }
    for f in &mut p.functions {
        let body = std::mem::replace(&mut f.body, Stmt::Empty);
        f.body = if level >= OptLevel::O2 {
            opt_stmt_local(opt_stmt(body))
        } else {
            opt_stmt(body)
        };
    }
    eliminate_unreachable_functions(&mut p);
    p
}

fn collect_expr_calls(e: &Expr, out: &mut Vec<String>) {
    match e {
        Expr::Call(name, args) => {
            out.push(name.clone());
            for a in args {
                collect_expr_calls(a, out);
            }
        }
        Expr::Unary(_, a) => collect_expr_calls(a, out),
        Expr::Binary(_, a, b) => {
            collect_expr_calls(a, out);
            collect_expr_calls(b, out);
        }
        _ => {}
    }
}

fn collect_stmt_calls(s: &Stmt, out: &mut Vec<String>) {
    match s {
        Stmt::Block(v) => {
            for x in v {
                collect_stmt_calls(x, out);
            }
        }
        Stmt::Var(v) => {
            if let Some(e) = &v.init {
                collect_expr_calls(e, out);
            }
        }
        Stmt::Assign(_, e) | Stmt::Expr(e) => collect_expr_calls(e, out),
        Stmt::If(c, a, b) => {
            collect_expr_calls(c, out);
            collect_stmt_calls(a, out);
            if let Some(b) = b {
                collect_stmt_calls(b, out);
            }
        }
        Stmt::While(c, b) => {
            collect_expr_calls(c, out);
            collect_stmt_calls(b, out);
        }
        Stmt::DoWhile(b, c) => {
            collect_stmt_calls(b, out);
            collect_expr_calls(c, out);
        }
        Stmt::For(i, c, st, b) => {
            if let Some(i) = i {
                collect_stmt_calls(i, out);
            }
            if let Some(c) = c {
                collect_expr_calls(c, out);
            }
            if let Some(st) = st {
                collect_stmt_calls(st, out);
            }
            collect_stmt_calls(b, out);
        }
        Stmt::Return(e) => {
            if let Some(e) = e {
                collect_expr_calls(e, out);
            }
        }
        Stmt::Break | Stmt::Continue | Stmt::Empty => {}
    }
}

/// Remove functions that cannot be reached from main().
///
/// SVM-C has no function pointers or separate linker, so reachability through
/// direct calls is complete. This is deliberately an optimization pass: O0
/// and svm-c-unopt-only retain every source/include function for pedagogical
/// comparison, while O1/O2/Os only emit code that the program can call.
pub(crate) fn eliminate_unreachable_functions(p: &mut Program) {
    let known: HashSet<String> = p.functions.iter().map(|f| f.name.clone()).collect();
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for f in &p.functions {
        let mut calls = Vec::new();
        collect_stmt_calls(&f.body, &mut calls);
        calls.retain(|n| known.contains(n));
        graph.insert(f.name.clone(), calls);
    }

    let mut live = HashSet::new();
    let mut work = vec![String::from("main")];
    while let Some(name) = work.pop() {
        if !live.insert(name.clone()) {
            continue;
        }
        if let Some(calls) = graph.get(&name) {
            for callee in calls {
                if !live.contains(callee) {
                    work.push(callee.clone());
                }
            }
        }
    }
    p.functions.retain(|f| live.contains(&f.name));
}

fn subst_expr(e: Expr, known: &HashMap<String, Expr>) -> Expr {
    match e {
        Expr::Var(n) => known.get(&n).cloned().unwrap_or(Expr::Var(n)),
        Expr::Unary(op, a) => opt_expr(Expr::Unary(op, Box::new(subst_expr(*a, known)))),
        Expr::Binary(op, a, b) => opt_expr(Expr::Binary(
            op,
            Box::new(subst_expr(*a, known)),
            Box::new(subst_expr(*b, known)),
        )),
        Expr::Call(n, args) => {
            Expr::Call(n, args.into_iter().map(|x| subst_expr(x, known)).collect())
        }
        x => x,
    }
}

fn forget_var(known: &mut HashMap<String, Expr>, name: &str) {
    known.remove(name);
    known.retain(|_, v| !matches!(v, Expr::Var(n) if n == name));
}

/// Return true when an expression reads `name`.
///
/// This is required by the local dead-store pass: in
///
///     x = a + b;
///     x = x + carry;
///
/// the first assignment is NOT dead because the second assignment consumes
/// its value.  Calls are traversed as well because an argument may read x.
fn expr_reads_var(e: &Expr, name: &str) -> bool {
    match e {
        Expr::Var(n) => n == name,
        Expr::Unary(_, a) => expr_reads_var(a, name),
        Expr::Binary(_, a, b) => expr_reads_var(a, name) || expr_reads_var(b, name),
        Expr::Call(_, args) => args.iter().any(|a| expr_reads_var(a, name)),
        _ => false,
    }
}

fn opt_block_local(v: Vec<Stmt>) -> Vec<Stmt> {
    let mut known: HashMap<String, Expr> = HashMap::new();
    let mut out: Vec<Stmt> = Vec::new();
    for s in v {
        match s {
            Stmt::Var(mut d) => {
                if let Some(e) = d.init.take() {
                    let e = subst_expr(e, &known);
                    let side = expr_has_call(&e);
                    d.init = Some(e.clone());
                    forget_var(&mut known, &d.name);
                    if !side && matches!(e, Expr::Num(_) | Expr::Var(_)) {
                        known.insert(d.name.clone(), e);
                    }
                    if side {
                        known.clear();
                    }
                }
                out.push(Stmt::Var(d));
            }
            Stmt::Assign(name, e) => {
                let e = subst_expr(e, &known);
                let side = expr_has_call(&e);
                // Cheap dead-store elimination: a directly overwritten assignment has no observable value.
                // A self-referential update (x = x + y) consumes the previous value,
                // so the preceding assignment/initializer must be retained.
                if !side && !expr_reads_var(&e, &name) {
                    let drop_prev = matches!(out.last(), Some(Stmt::Assign(prev, pe)) if prev == &name && !expr_has_call(pe));
                    if drop_prev {
                        out.pop();
                    } else if let Some(Stmt::Var(prev)) = out.last_mut() {
                        if prev.name == name
                            && prev
                                .init
                                .as_ref()
                                .map(|x| !expr_has_call(x))
                                .unwrap_or(false)
                        {
                            prev.init = None;
                        }
                    }
                }
                forget_var(&mut known, &name);
                if !side && matches!(e, Expr::Num(_) | Expr::Var(_)) {
                    known.insert(name.clone(), e.clone());
                }
                out.push(Stmt::Assign(name, e));
                if side {
                    known.clear();
                }
            }
            Stmt::Expr(e) => {
                let e = subst_expr(e, &known);
                if expr_has_call(&e) {
                    known.clear();
                }
                out.push(Stmt::Expr(e));
            }
            Stmt::Return(e) => {
                let e = e.map(|x| subst_expr(x, &known));
                out.push(Stmt::Return(e));
                known.clear();
            }
            Stmt::If(c, a, b) => {
                let c = subst_expr(c, &known);
                known.clear();
                out.push(opt_stmt(Stmt::If(
                    c,
                    Box::new(opt_stmt_local(*a)),
                    b.map(|x| Box::new(opt_stmt_local(*x))),
                )));
            }
            Stmt::While(c, b) => {
                // Do not substitute block-local known values into a loop condition.
                // The loop body may modify those variables before the condition is
                // evaluated again (e.g. i = 0; while (i < 16) { i = i + 1; }).
                // Constant/algebraic folding inside the condition is still safe.
                let c = opt_expr(c);
                known.clear();
                out.push(opt_stmt(Stmt::While(c, Box::new(opt_stmt_local(*b)))));
            }
            Stmt::DoWhile(b, c) => {
                known.clear();
                out.push(opt_stmt(Stmt::DoWhile(
                    Box::new(opt_stmt_local(*b)),
                    opt_expr(c),
                )));
            }
            Stmt::For(i, c, st, b) => {
                let i = i.map(|x| Box::new(opt_stmt_local(*x)));
                // Same rule as while: values known before the loop cannot be
                // propagated into a condition that is re-evaluated after body/step.
                let c = c.map(opt_expr);
                known.clear();
                let st = st.map(|x| Box::new(opt_stmt_local(*x)));
                out.push(opt_stmt(Stmt::For(i, c, st, Box::new(opt_stmt_local(*b)))));
            }
            other => {
                known.clear();
                out.push(opt_stmt_local(other));
            }
        }
    }
    out
}

fn opt_stmt_local(s: Stmt) -> Stmt {
    match s {
        Stmt::Block(v) => Stmt::Block(opt_block_local(v)),
        Stmt::If(c, a, b) => Stmt::If(
            c,
            Box::new(opt_stmt_local(*a)),
            b.map(|x| Box::new(opt_stmt_local(*x))),
        ),
        Stmt::While(c, b) => Stmt::While(c, Box::new(opt_stmt_local(*b))),
        Stmt::DoWhile(b, c) => Stmt::DoWhile(Box::new(opt_stmt_local(*b)), c),
        Stmt::For(i, c, st, b) => Stmt::For(
            i.map(|x| Box::new(opt_stmt_local(*x))),
            c,
            st.map(|x| Box::new(opt_stmt_local(*x))),
            Box::new(opt_stmt_local(*b)),
        ),
        x => x,
    }
}

fn opt_stmt(s: Stmt) -> Stmt {
    match s {
        Stmt::Block(v) => {
            let mut out = Vec::new();
            for x in v {
                let x = opt_stmt(x);
                if !matches!(x, Stmt::Empty) {
                    out.push(x);
                }
            }
            Stmt::Block(out)
        }
        Stmt::Var(mut v) => {
            v.init = v.init.map(opt_expr);
            Stmt::Var(v)
        }
        Stmt::Assign(n, e) => Stmt::Assign(n, opt_expr(e)),
        Stmt::Expr(e) => Stmt::Expr(opt_expr(e)),
        Stmt::If(c, a, b) => {
            let c = opt_expr(c);
            let a = Box::new(opt_stmt(*a));
            let b = b.map(|x| Box::new(opt_stmt(*x)));
            match c {
                Expr::Num(0) => b.map(|x| *x).unwrap_or(Stmt::Empty),
                Expr::Num(_) => *a,
                _ => Stmt::If(c, a, b),
            }
        }
        Stmt::While(c, b) => {
            let c = opt_expr(c);
            let b = Box::new(opt_stmt(*b));
            if matches!(&c, Expr::Num(0)) {
                Stmt::Empty
            } else {
                Stmt::While(c, b)
            }
        }
        Stmt::DoWhile(b, c) => Stmt::DoWhile(Box::new(opt_stmt(*b)), opt_expr(c)),
        Stmt::Break => Stmt::Break,
        Stmt::Continue => Stmt::Continue,
        Stmt::For(i, c, st, b) => {
            let i = i.map(|x| Box::new(opt_stmt(*x)));
            let c = c.map(opt_expr);
            let st = st.map(|x| Box::new(opt_stmt(*x)));
            let b = Box::new(opt_stmt(*b));
            if matches!(&c, Some(Expr::Num(0))) {
                i.map(|x| *x).unwrap_or(Stmt::Empty)
            } else {
                Stmt::For(i, c, st, b)
            }
        }
        Stmt::Return(e) => Stmt::Return(e.map(opt_expr)),
        Stmt::Empty => Stmt::Empty,
    }
}

fn opt_expr(e: Expr) -> Expr {
    match e {
        Expr::Unary(op, a) => {
            let a = opt_expr(*a);
            if let Expr::Num(v) = &a {
                let v = *v;
                let r = match op {
                    UnOp::Neg => v.wrapping_neg(),
                    UnOp::Not => !v,
                    UnOp::LogicalNot => (v == 0) as u16,
                    UnOp::Inc1 => v.wrapping_add(1),
                    UnOp::Dec1 => v.wrapping_sub(1),
                    UnOp::Shl1 => v.wrapping_shl(1),
                    UnOp::Shr1 => v >> 1,
                };
                Expr::Num(r)
            } else {
                Expr::Unary(op, Box::new(a))
            }
        }
        Expr::Binary(op, a, b) => {
            let a = opt_expr(*a);
            let b = opt_expr(*b);
            if let (Expr::Num(x), Expr::Num(y)) = (&a, &b) {
                if let Some(v) = fold_bin(op, *x, *y) {
                    return Expr::Num(v);
                }
            }
            match (op, &a, &b) {
                (BinOp::Add, _, Expr::Num(0))
                | (BinOp::Sub, _, Expr::Num(0))
                | (BinOp::Or, _, Expr::Num(0))
                | (BinOp::Xor, _, Expr::Num(0))
                | (BinOp::Shl, _, Expr::Num(0))
                | (BinOp::Shr, _, Expr::Num(0)) => a.clone(),
                (BinOp::Add, Expr::Num(0), _) => b.clone(),
                (BinOp::Mul, _, Expr::Num(1)) | (BinOp::Div, _, Expr::Num(1)) => a.clone(),
                (BinOp::Mul, Expr::Num(1), _) => b.clone(),
                (BinOp::And, _, Expr::Num(0xFFFF)) => a.clone(),
                (BinOp::And, _, Expr::Num(0)) if !expr_has_call(&a) => Expr::Num(0),
                (BinOp::Mul, _, Expr::Num(0)) if !expr_has_call(&a) => Expr::Num(0),
                (BinOp::Mul, Expr::Num(0), _) if !expr_has_call(&b) => Expr::Num(0),
                (BinOp::Add, _, Expr::Num(1)) => Expr::Unary(UnOp::Inc1, Box::new(a.clone())),
                (BinOp::Sub, _, Expr::Num(1)) => Expr::Unary(UnOp::Dec1, Box::new(a.clone())),
                (BinOp::Mul, _, Expr::Num(2)) => Expr::Unary(UnOp::Shl1, Box::new(a.clone())),
                (BinOp::Mul, Expr::Num(2), _) => Expr::Unary(UnOp::Shl1, Box::new(b.clone())),
                (BinOp::Shl, _, Expr::Num(1)) => Expr::Unary(UnOp::Shl1, Box::new(a.clone())),
                (BinOp::Shr, _, Expr::Num(1)) => Expr::Unary(UnOp::Shr1, Box::new(a.clone())),
                (BinOp::Mul, _, Expr::Num(v)) if v.is_power_of_two() => Expr::Binary(
                    BinOp::Shl,
                    Box::new(a.clone()),
                    Box::new(Expr::Num(v.trailing_zeros() as u16)),
                ),
                (BinOp::Div, _, Expr::Num(v)) if *v > 1 && v.is_power_of_two() => Expr::Binary(
                    BinOp::Shr,
                    Box::new(a.clone()),
                    Box::new(Expr::Num(v.trailing_zeros() as u16)),
                ),
                (BinOp::Mod, _, Expr::Num(v)) if *v > 1 && v.is_power_of_two() => {
                    Expr::Binary(BinOp::And, Box::new(a.clone()), Box::new(Expr::Num(*v - 1)))
                }
                _ => Expr::Binary(op, Box::new(a), Box::new(b)),
            }
        }
        Expr::Call(n, a) => Expr::Call(n, a.into_iter().map(opt_expr).collect()),
        x => x,
    }
}

fn fold_bin(op: BinOp, a: u16, b: u16) -> Option<u16> {
    Some(match op {
        BinOp::LogicalOr => ((a != 0) || (b != 0)) as u16,
        BinOp::LogicalAnd => ((a != 0) && (b != 0)) as u16,
        BinOp::Add => a.wrapping_add(b),
        BinOp::Sub => a.wrapping_sub(b),
        BinOp::Mul => a.wrapping_mul(b),
        BinOp::Div => {
            if b == 0 {
                return None;
            } else {
                a / b
            }
        }
        BinOp::Mod => {
            if b == 0 {
                return None;
            } else {
                a % b
            }
        }
        BinOp::And => a & b,
        BinOp::Or => a | b,
        BinOp::Xor => a ^ b,
        BinOp::Shl => a.wrapping_shl((b & 15) as u32),
        BinOp::Shr => a.wrapping_shr((b & 15) as u32),
        BinOp::Eq => (a == b) as u16,
        BinOp::Ne => (a != b) as u16,
        BinOp::Lt => (a < b) as u16,
        BinOp::Gt => (a > b) as u16,
        BinOp::Le => (a <= b) as u16,
        BinOp::Ge => (a >= b) as u16,
    })
}

#[cfg(test)]
mod local_dse_regression_tests {
    use super::*;

    #[test]
    fn keeps_assignment_consumed_by_self_update() {
        let body = Stmt::Block(vec![
            Stmt::Assign(
                "x".into(),
                Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Var("a".into())),
                    Box::new(Expr::Var("b".into())),
                ),
            ),
            Stmt::Assign(
                "x".into(),
                Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Var("x".into())),
                    Box::new(Expr::Var("carry".into())),
                ),
            ),
        ]);
        let optimized = opt_stmt_local(body);
        match optimized {
            Stmt::Block(v) => assert_eq!(v.len(), 2),
            _ => panic!("expected block"),
        }
    }

    #[test]
    fn keeps_initializer_consumed_by_self_update() {
        let body = Stmt::Block(vec![
            Stmt::Var(VarDecl {
                ty: Ty::U16,
                name: "x".into(),
                len: 1,
                init: Some(Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Var("a".into())),
                    Box::new(Expr::Var("b".into())),
                )),
            }),
            Stmt::Assign(
                "x".into(),
                Expr::Binary(
                    BinOp::Add,
                    Box::new(Expr::Var("x".into())),
                    Box::new(Expr::Var("carry".into())),
                ),
            ),
        ]);
        let optimized = opt_stmt_local(body);
        match optimized {
            Stmt::Block(v) => match &v[0] {
                Stmt::Var(d) => assert!(d.init.is_some()),
                _ => panic!("expected variable declaration"),
            },
            _ => panic!("expected block"),
        }
    }

    #[test]
    fn still_removes_true_direct_overwrite() {
        let body = Stmt::Block(vec![
            Stmt::Assign("x".into(), Expr::Num(5)),
            Stmt::Assign("x".into(), Expr::Num(7)),
        ]);
        let optimized = opt_stmt_local(body);
        match optimized {
            Stmt::Block(v) => {
                assert_eq!(v.len(), 1);
                assert!(matches!(&v[0], Stmt::Assign(n, Expr::Num(7)) if n == "x"));
            }
            _ => panic!("expected block"),
        }
    }
}
