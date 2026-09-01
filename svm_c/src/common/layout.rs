use crate::model::*;
use std::collections::HashMap;

fn alloc_static_n(
    next: &mut u16,
    ty: Ty,
    len: u16,
    memreg_hot_scratch: bool,
) -> Result<u16, String> {
    let bytes = size(ty).checked_mul(len).ok_or("array size overflow")?;
    // MemReg reserves 0x000E..0x000F as a compiler-owned 16-bit expression scratch.
    // Keep 0x0000..0x000D available to user data, then continue at 0x0010.
    if memreg_hot_scratch {
        const SCRATCH: u16 = 0x000E;
        const AFTER_SCRATCH: u16 = 0x0010;
        if *next < SCRATCH && (*next as u32 + bytes as u32) > SCRATCH as u32 {
            *next = AFTER_SCRATCH;
        } else if *next >= SCRATCH && *next < AFTER_SCRATCH {
            *next = AFTER_SCRATCH;
        }
    }
    if *next < 0x00F0 && (*next as u32 + bytes as u32) > 0x00F0 {
        *next = 0xE000;
    }
    if *next == 0x00F0 {
        *next = 0xE000;
    }
    let addr = *next;
    *next = next.checked_add(bytes).ok_or("data address overflow")?;
    if *next > 0x00F0 && *next < 0xE000 {
        return Err("internal static allocator gap".into());
    }
    if *next > 0xFB00 {
        return Err(
            "static variables exceed zero page plus upper 0xE000..0xFAFF RAM data area".into(),
        );
    }
    Ok(addr)
}

pub(crate) fn make_layout(p: &Program, target: Target) -> Result<Layout, String> {
    let mut next = if matches!(target, Target::Memory2Memory | Target::Belt) {
        0x0020u16
    } else {
        0x0000u16
    };
    let memreg_hot_scratch = target == Target::MemReg;
    let mut globals = HashMap::new();
    let mut locals = HashMap::new();
    let mut funcs = HashMap::new();
    for g in &p.globals {
        let a = alloc_static_n(&mut next, g.ty, g.len, memreg_hot_scratch)?;
        globals.insert(
            g.name.clone(),
            VarInfo {
                ty: g.ty,
                addr: a,
                len: g.len,
            },
        );
    }
    for f in &p.functions {
        funcs.insert(
            f.name.clone(),
            FunctionSig {
                ret: f.ret,
                params: f.params.iter().map(|p| p.ty).collect(),
            },
        );
        for a in &f.params {
            let ad = alloc_static_n(&mut next, a.ty, 1, memreg_hot_scratch)?;
            locals.insert(
                (f.name.clone(), a.name.clone()),
                VarInfo {
                    ty: a.ty,
                    addr: ad,
                    len: 1,
                },
            );
        }
        if f.extern_asm && f.ret != Ty::Void {
            let ad = alloc_static_n(&mut next, f.ret, 1, memreg_hot_scratch)?;
            locals.insert(
                (f.name.clone(), String::from("#asm_return")),
                VarInfo {
                    ty: f.ret,
                    addr: ad,
                    len: 1,
                },
            );
        }
        alloc_stmt(&f.name, &f.body, &mut next, &mut locals, memreg_hot_scratch)?;
    }

    Ok(Layout {
        globals,
        locals,
        funcs,
    })
}
fn alloc_stmt(
    fun: &str,
    s: &Stmt,
    next: &mut u16,
    map: &mut HashMap<(String, String), VarInfo>,
    memreg_hot_scratch: bool,
) -> Result<(), String> {
    match s {
        Stmt::Var(v) => {
            let a = alloc_static_n(next, v.ty, v.len, memreg_hot_scratch)?;
            map.insert(
                (fun.to_string(), v.name.clone()),
                VarInfo {
                    ty: v.ty,
                    addr: a,
                    len: v.len,
                },
            );
        }
        Stmt::Block(v) => {
            for x in v {
                alloc_stmt(fun, x, next, map, memreg_hot_scratch)?
            }
        }
        Stmt::If(_, a, b) => {
            alloc_stmt(fun, a, next, map, memreg_hot_scratch)?;
            if let Some(b) = b {
                alloc_stmt(fun, b, next, map, memreg_hot_scratch)?
            }
        }
        Stmt::While(_, b) | Stmt::DoWhile(b, _) => {
            alloc_stmt(fun, b, next, map, memreg_hot_scratch)?
        }
        Stmt::For(a, _, c, b) => {
            if let Some(a) = a {
                alloc_stmt(fun, a, next, map, memreg_hot_scratch)?
            };
            if let Some(c) = c {
                alloc_stmt(fun, c, next, map, memreg_hot_scratch)?
            };
            alloc_stmt(fun, b, next, map, memreg_hot_scratch)?
        }
        _ => {}
    }
    Ok(())
}
fn size(t: Ty) -> u16 {
    t.size()
}

impl Layout {
    pub fn var(&self, fun: &str, name: &str) -> Option<&VarInfo> {
        self.locals
            .get(&(fun.to_string(), name.to_string()))
            .or_else(|| self.globals.get(name))
    }
}

pub(crate) fn lower_subset_plus(mut p: Program, layout: &Layout) -> Result<Program, String> {
    fn lower_expr(fun: &str, e: Expr, layout: &Layout) -> Result<Expr, String> {
        match e {
            Expr::SizeOfName(n) => {
                let v = layout
                    .var(fun, &n)
                    .ok_or_else(|| format!("unknown object '{}' in sizeof", n))?;
                Ok(Expr::Num(size(v.ty).saturating_mul(v.len)))
            }
            Expr::AddrOf(n) => {
                let v = layout
                    .var(fun, &n)
                    .ok_or_else(|| format!("unknown object '{}' in address-of", n))?;
                Ok(Expr::Num(v.addr))
            }
            Expr::Str(x) => Ok(Expr::Str(x)),
            Expr::Var(n) => {
                if let Some(v) = layout.var(fun, &n) {
                    if v.len > 1 {
                        return Ok(Expr::Num(v.addr));
                    }
                }
                Ok(Expr::Var(n))
            }
            Expr::Unary(op, a) => Ok(Expr::Unary(op, Box::new(lower_expr(fun, *a, layout)?))),
            Expr::Binary(op, a, b) => Ok(Expr::Binary(
                op,
                Box::new(lower_expr(fun, *a, layout)?),
                Box::new(lower_expr(fun, *b, layout)?),
            )),
            Expr::Call(n, args) => {
                if let Some(name) = n.strip_prefix("__index__") {
                    let v = layout
                        .var(fun, name)
                        .ok_or_else(|| format!("unknown array '{}'", name))?;
                    if v.len <= 1 {
                        return Err(format!("'{}' is not an array", name));
                    }
                    if args.len() != 1 {
                        return Err("internal array index arity error".into());
                    }
                    let idx0 = args.into_iter().next().unwrap();
                    if let Expr::Num(i) = &idx0 {
                        if *i >= v.len {
                            return Err(format!(
                                "constant index {} is out of bounds for {}[{}]",
                                i, name, v.len
                            ));
                        }
                    }
                    let idx = lower_expr(fun, idx0, layout)?;
                    let off = if size(v.ty) == 1 {
                        idx
                    } else {
                        Expr::Binary(BinOp::Mul, Box::new(idx), Box::new(Expr::Num(size(v.ty))))
                    };
                    let addr = Expr::Binary(BinOp::Add, Box::new(Expr::Num(v.addr)), Box::new(off));
                    let op = match v.ty.size() {
                        1 => "load8",
                        2 => "load16",
                        _ => {
                            return Err(format!(
                                "wide array element '{}' must be accessed through its address",
                                name
                            ));
                        }
                    };
                    return Ok(Expr::Call(op.into(), vec![addr]));
                }
                if let Some(name) = n.strip_prefix("__store__") {
                    let v = layout
                        .var(fun, name)
                        .ok_or_else(|| format!("unknown array '{}'", name))?;
                    if v.len <= 1 {
                        return Err(format!("'{}' is not an array", name));
                    }
                    if args.len() != 2 {
                        return Err("internal array store arity error".into());
                    }
                    let mut it = args.into_iter();
                    let idx0 = it.next().unwrap();
                    let val = it.next().unwrap();
                    if let Expr::Num(i) = &idx0 {
                        if *i >= v.len {
                            return Err(format!(
                                "constant index {} is out of bounds for {}[{}]",
                                i, name, v.len
                            ));
                        }
                    }
                    let idx = lower_expr(fun, idx0, layout)?;
                    let val = lower_expr(fun, val, layout)?;
                    let off = if size(v.ty) == 1 {
                        idx
                    } else {
                        Expr::Binary(BinOp::Mul, Box::new(idx), Box::new(Expr::Num(size(v.ty))))
                    };
                    let addr = Expr::Binary(BinOp::Add, Box::new(Expr::Num(v.addr)), Box::new(off));
                    let op = match v.ty.size() {
                        1 => "store8",
                        2 => "store16",
                        _ => {
                            return Err(format!(
                                "wide array element '{}' must be accessed through its address",
                                name
                            ));
                        }
                    };
                    return Ok(Expr::Call(op.into(), vec![addr, val]));
                }
                Ok(Expr::Call(
                    n,
                    args.into_iter()
                        .map(|x| lower_expr(fun, x, layout))
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            }
            x => Ok(x),
        }
    }
    fn lower_stmt(fun: &str, s: Stmt, layout: &Layout) -> Result<Stmt, String> {
        Ok(match s {
            Stmt::Block(v) => Stmt::Block(
                v.into_iter()
                    .map(|x| lower_stmt(fun, x, layout))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Stmt::Var(mut v) => {
                if let Some(e) = v.init.take() {
                    v.init = Some(lower_expr(fun, e, layout)?);
                }
                Stmt::Var(v)
            }
            Stmt::Assign(n, e) => {
                if layout.var(fun, &n).map(|v| v.len > 1).unwrap_or(false) {
                    return Err(format!("cannot assign to array '{}' without an index", n));
                }
                Stmt::Assign(n, lower_expr(fun, e, layout)?)
            }
            Stmt::Expr(e) => Stmt::Expr(lower_expr(fun, e, layout)?),
            Stmt::If(c, a, b) => Stmt::If(
                lower_expr(fun, c, layout)?,
                Box::new(lower_stmt(fun, *a, layout)?),
                match b {
                    Some(x) => Some(Box::new(lower_stmt(fun, *x, layout)?)),
                    None => None,
                },
            ),
            Stmt::While(c, b) => Stmt::While(
                lower_expr(fun, c, layout)?,
                Box::new(lower_stmt(fun, *b, layout)?),
            ),
            Stmt::DoWhile(b, c) => Stmt::DoWhile(
                Box::new(lower_stmt(fun, *b, layout)?),
                lower_expr(fun, c, layout)?,
            ),
            Stmt::Break => Stmt::Break,
            Stmt::Continue => Stmt::Continue,
            Stmt::For(a, c, st, b) => Stmt::For(
                match a {
                    Some(x) => Some(Box::new(lower_stmt(fun, *x, layout)?)),
                    None => None,
                },
                match c {
                    Some(x) => Some(lower_expr(fun, x, layout)?),
                    None => None,
                },
                match st {
                    Some(x) => Some(Box::new(lower_stmt(fun, *x, layout)?)),
                    None => None,
                },
                Box::new(lower_stmt(fun, *b, layout)?),
            ),
            Stmt::Return(e) => Stmt::Return(match e {
                Some(x) => Some(lower_expr(fun, x, layout)?),
                None => None,
            }),
            Stmt::Empty => Stmt::Empty,
        })
    }
    for g in &mut p.globals {
        if let Some(e) = g.init.take() {
            g.init = Some(lower_expr("", e, layout)?);
        }
    }
    for f in &mut p.functions {
        let body = std::mem::replace(&mut f.body, Stmt::Empty);
        f.body = lower_stmt(&f.name, body, layout)?;
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_overflow_uses_upper_ram_not_code_space() {
        let mut next = 0x00EEu16;
        let a = alloc_static_n(&mut next, Ty::U32, 1, false).unwrap();
        assert_eq!(a, 0xE000);
        assert_eq!(next, 0xE004);
    }

    #[test]
    fn upper_static_region_stops_before_stack_area() {
        let mut next = 0xFAFE;
        assert!(alloc_static_n(&mut next, Ty::U32, 1, false).is_err());
    }
}
