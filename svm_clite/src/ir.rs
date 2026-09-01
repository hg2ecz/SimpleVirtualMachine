use std::collections::HashMap;

use crate::ast::*;

/// Tiny target-neutral assembly-like IR.
///
/// CLIR has virtual temporaries and labels, but no physical registers,
/// stack opcodes, belt positions, accumulator state, or target addressing.
pub fn emit_ir(program: &Program) -> Result<String, String> {
    let functions: HashMap<String, &Function> = program
        .functions
        .iter()
        .map(|function| (function.name.clone(), function))
        .collect();
    let globals: HashMap<String, Ty> = program
        .globals
        .iter()
        .map(|global| (global.name.clone(), global.ty.clone()))
        .collect();

    let mut out = String::new();
    out.push_str("; SVM C-Lite target-neutral IR (CLIR 0.1)\n");
    out.push_str("; virtual temporaries: %0, %1, ...\n\n");

    for global in &program.globals {
        match &global.ty {
            Ty::Array(elem, count) => {
                out.push_str(&format!(
                    "global {} {}[{}]\n",
                    elem.name(),
                    global.name,
                    count
                ));
            }
            _ => {
                out.push_str(&format!("global {} {}", global.ty.name(), global.name));
                match &global.init {
                    Some(Expr::Int(value)) => out.push_str(&format!(" = {value}")),
                    Some(Expr::Bool(value)) => {
                        out.push_str(if *value { " = 1" } else { " = 0" });
                    }
                    _ => {}
                }
                out.push('\n');
            }
        }
    }

    if !program.globals.is_empty() {
        out.push('\n');
    }

    for function in &program.functions {
        let mut emitter = Emitter::new(&functions, globals.clone(), function);
        emitter.function(function)?;
        out.push_str(&emitter.out);
        out.push('\n');
    }

    Ok(out)
}

struct Emitter<'a> {
    out: String,
    next_tmp: usize,
    next_label: usize,
    functions: &'a HashMap<String, &'a Function>,
    env: HashMap<String, Ty>,
    loops: Vec<(String, String)>, // break, continue
}

impl<'a> Emitter<'a> {
    fn new(
        functions: &'a HashMap<String, &'a Function>,
        mut env: HashMap<String, Ty>,
        function: &Function,
    ) -> Self {
        for param in &function.params {
            env.insert(param.name.clone(), param.ty.clone());
        }
        Self {
            out: String::new(),
            next_tmp: 0,
            next_label: 0,
            functions,
            env,
            loops: Vec::new(),
        }
    }

    fn line(&mut self, text: impl AsRef<str>) {
        self.out.push_str("  ");
        self.out.push_str(text.as_ref());
        self.out.push('\n');
    }

    fn tmp(&mut self) -> String {
        let temp = format!("%{}", self.next_tmp);
        self.next_tmp += 1;
        temp
    }

    fn label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.next_label);
        self.next_label += 1;
        label
    }

    fn function(&mut self, function: &Function) -> Result<(), String> {
        let params = function
            .params
            .iter()
            .map(|param| format!("{} {}", param.ty.name(), param.name))
            .collect::<Vec<_>>()
            .join(", ");

        self.out.push_str(&format!(
            "fn {}({}) -> {}\n",
            function.name,
            params,
            function.ret.name()
        ));
        for statement in &function.body {
            self.stmt(statement)?;
        }
        self.out.push_str("end\n");
        Ok(())
    }

    fn stmt(&mut self, statement: &Stmt) -> Result<(), String> {
        match statement {
            Stmt::Let { name, ty, init } => {
                match ty {
                    Ty::Array(elem, count) => {
                        self.line(format!("local {} {}[{}]", elem.name(), name, count));
                    }
                    _ => self.line(format!("local {} {}", ty.name(), name)),
                }
                self.env.insert(name.clone(), ty.clone());
                if let Some(expr) = init {
                    let value = self.expr(expr)?;
                    self.line(format!("store.{} {}, {}", ty.clir_suffix(), name, value));
                }
            }
            Stmt::Assign { lhs, rhs } => {
                let value = self.expr(rhs)?;
                self.store_lvalue(lhs, &value)?;
            }
            Stmt::Expr(expr) => {
                let value = self.expr(expr)?;
                if value != "void" {
                    self.line(format!("drop {value}"));
                }
            }
            Stmt::Return(None) => self.line("ret"),
            Stmt::Return(Some(expr)) => {
                let value = self.expr(expr)?;
                self.line(format!("ret {value}"));
            }
            Stmt::Break => {
                let label = self
                    .loops
                    .last()
                    .map(|labels| labels.0.clone())
                    .ok_or_else(|| String::from("break outside loop"))?;
                self.line(format!("jmp {label}"));
            }
            Stmt::Continue => {
                let label = self
                    .loops
                    .last()
                    .map(|labels| labels.1.clone())
                    .ok_or_else(|| String::from("continue outside loop"))?;
                self.line(format!("jmp {label}"));
            }
            Stmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let else_label = self.label("if_else");
                let end_label = self.label("if_end");
                let condition = self.expr(cond)?;
                self.line(format!("jz {condition}, {else_label}"));

                for statement in then_body {
                    self.stmt(statement)?;
                }
                self.line(format!("jmp {end_label}"));
                self.out.push_str(&format!("{else_label}:\n"));

                for statement in else_body {
                    self.stmt(statement)?;
                }
                self.out.push_str(&format!("{end_label}:\n"));
            }
            Stmt::While { cond, body } => {
                let test_label = self.label("while_test");
                let end_label = self.label("while_end");

                self.out.push_str(&format!("{test_label}:\n"));
                let condition = self.expr(cond)?;
                self.line(format!("jz {condition}, {end_label}"));

                self.loops.push((end_label.clone(), test_label.clone()));
                for statement in body {
                    self.stmt(statement)?;
                }
                self.loops.pop();

                self.line(format!("jmp {test_label}"));
                self.out.push_str(&format!("{end_label}:\n"));
            }
        }
        Ok(())
    }

    fn store_lvalue(&mut self, lhs: &Expr, value: &str) -> Result<(), String> {
        match lhs {
            Expr::Var(name) => {
                let ty = self
                    .env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("unknown variable {name}"))?;
                self.line(format!("store.{} {}, {}", ty.clir_suffix(), name, value));
            }
            Expr::Unary {
                op: UnaryOp::Deref,
                expr: pointer,
            } => {
                let Ty::Ptr(elem) = self.expr_ty(pointer)? else {
                    return Err("dereference assignment requires pointer".into());
                };
                let address = self.expr(pointer)?;
                self.line(format!(
                    "storemem.{} {}, {}",
                    elem.clir_suffix(),
                    address,
                    value
                ));
            }
            Expr::Index { base, index } => {
                let (address, ty) = self.index_addr(base, index)?;
                self.line(format!(
                    "storemem.{} {}, {}",
                    ty.clir_suffix(),
                    address,
                    value
                ));
            }
            _ => return Err("left side is not assignable".into()),
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Bool(value) => {
                let temp = self.tmp();
                self.line(format!(
                    "const.bool {}, {}",
                    temp,
                    if *value { 1 } else { 0 }
                ));
                Ok(temp)
            }
            Expr::Int(value) => {
                let temp = self.tmp();
                self.line(format!("const.u16 {temp}, {value}"));
                Ok(temp)
            }
            Expr::Var(name) => {
                let ty = self
                    .env
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("unknown variable {name}"))?;
                if matches!(ty, Ty::Array(_, _)) {
                    return Err("array value requires indexing or address-of".into());
                }
                let temp = self.tmp();
                self.line(format!("load.{} {}, {}", ty.clir_suffix(), temp, name));
                Ok(temp)
            }
            Expr::Unary {
                op: UnaryOp::AddrOf,
                expr,
            } => self.addr_of(expr),
            Expr::Unary {
                op: UnaryOp::Deref,
                expr: pointer,
            } => {
                let Ty::Ptr(elem) = self.expr_ty(pointer)? else {
                    return Err("dereference requires pointer".into());
                };
                let address = self.expr(pointer)?;
                let temp = self.tmp();
                self.line(format!(
                    "loadmem.{} {}, {}",
                    elem.clir_suffix(),
                    temp,
                    address
                ));
                Ok(temp)
            }
            Expr::Unary { op, expr } => {
                let source = self.expr(expr)?;
                let ty = self.expr_ty(expr)?;
                let temp = self.tmp();
                let op_name = match op {
                    UnaryOp::Neg => "neg",
                    UnaryOp::BitNot => "not",
                    _ => unreachable!(),
                };
                self.line(format!(
                    "{}.{} {}, {}",
                    op_name,
                    ty.clir_suffix(),
                    temp,
                    source
                ));
                Ok(temp)
            }
            Expr::Index { base, index } => {
                let (address, ty) = self.index_addr(base, index)?;
                let temp = self.tmp();
                self.line(format!(
                    "loadmem.{} {}, {}",
                    ty.clir_suffix(),
                    temp,
                    address
                ));
                Ok(temp)
            }
            Expr::Binary { op, lhs, rhs } => {
                let left = self.expr(lhs)?;
                let right = self.expr(rhs)?;
                // Comparisons produce bool, while the operation suffix describes
                // the operand interpretation (for example lt.i16).
                let op_ty = if is_comparison(*op) {
                    self.expr_ty(lhs)?
                } else {
                    self.expr_ty(expr)?
                };
                let temp = self.tmp();
                self.line(format!(
                    "{}.{} {}, {}, {}",
                    binary_name(*op),
                    op_ty.clir_suffix(),
                    temp,
                    left,
                    right
                ));
                Ok(temp)
            }
            Expr::Call { name, args } => self.call(name, args),
        }
    }

    fn call(&mut self, name: &str, args: &[Expr]) -> Result<String, String> {
        if matches!(name, "load8" | "load16" | "vload8" | "vload16") {
            let address = self.expr(&args[0])?;
            let temp = self.tmp();
            let volatile = if name.starts_with('v') { "v" } else { "" };
            let width = if name.ends_with('8') { "u8" } else { "u16" };
            self.line(format!("loadmem{volatile}.{width} {temp}, {address}"));
            return Ok(temp);
        }

        if matches!(name, "store8" | "store16" | "vstore8" | "vstore16") {
            let address = self.expr(&args[0])?;
            let value = self.expr(&args[1])?;
            let volatile = if name.starts_with('v') { "v" } else { "" };
            let width = if name.ends_with('8') { "u8" } else { "u16" };
            self.line(format!("storemem{volatile}.{width} {address}, {value}"));
            return Ok(String::from("void"));
        }

        let ret = self
            .functions
            .get(name)
            .ok_or_else(|| format!("unknown function {name}"))?
            .ret
            .clone();
        let mut lowered_args = Vec::new();
        for arg in args {
            lowered_args.push(self.expr(arg)?);
        }

        if ret == Ty::Void {
            self.line(format!("call {}({})", name, lowered_args.join(", ")));
            Ok(String::from("void"))
        } else {
            let temp = self.tmp();
            self.line(format!(
                "call.{} {} = {}({})",
                ret.clir_suffix(),
                temp,
                name,
                lowered_args.join(", ")
            ));
            Ok(temp)
        }
    }

    fn addr_of(&mut self, expr: &Expr) -> Result<String, String> {
        match expr {
            Expr::Var(name) => {
                self.env
                    .get(name)
                    .ok_or_else(|| format!("unknown variable {name}"))?;
                let temp = self.tmp();
                self.line(format!("addr {temp}, {name}"));
                Ok(temp)
            }
            Expr::Index { base, index } => {
                let (address, _) = self.index_addr(base, index)?;
                Ok(address)
            }
            _ => Err("address-of requires variable or indexed element".into()),
        }
    }

    fn index_addr(&mut self, base: &Expr, index: &Expr) -> Result<(String, Ty), String> {
        let base_ty = self.expr_ty(base)?;
        let (base_addr, elem) = match base_ty {
            Ty::Array(elem, _) => {
                let Expr::Var(name) = base else {
                    return Err("array base must be named".into());
                };
                let temp = self.tmp();
                self.line(format!("addr {temp}, {name}"));
                (temp, *elem)
            }
            Ty::Ptr(elem) => (self.expr(base)?, *elem),
            _ => return Err("indexing requires array or pointer".into()),
        };

        let index = self.expr(index)?;
        let address = self.tmp();
        self.line(format!(
            "index {}, {}, {}, {}",
            address,
            base_addr,
            index,
            elem.width()
        ));
        Ok((address, elem))
    }

    fn expr_ty(&self, expr: &Expr) -> Result<Ty, String> {
        match expr {
            Expr::Bool(_) => Ok(Ty::Bool),
            Expr::Int(_) => Ok(Ty::U16),
            Expr::Var(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| format!("unknown variable {name}")),
            Expr::Unary {
                op: UnaryOp::AddrOf,
                expr,
            } => match self.expr_ty(expr)? {
                Ty::Array(elem, _) => Ok(Ty::Ptr(elem)),
                other => Ok(Ty::Ptr(Box::new(other))),
            },
            Expr::Unary {
                op: UnaryOp::Deref,
                expr,
            } => match self.expr_ty(expr)? {
                Ty::Ptr(elem) => Ok(*elem),
                _ => Err("dereference requires pointer".into()),
            },
            Expr::Unary { expr, .. } => self.expr_ty(expr),
            Expr::Index { base, .. } => match self.expr_ty(base)? {
                Ty::Array(elem, _) | Ty::Ptr(elem) => Ok(*elem),
                _ => Err("indexing requires array or pointer".into()),
            },
            Expr::Call { name, .. } => {
                if matches!(name.as_str(), "load8" | "vload8") {
                    Ok(Ty::U8)
                } else if matches!(name.as_str(), "load16" | "vload16") {
                    Ok(Ty::U16)
                } else if matches!(name.as_str(), "store8" | "store16" | "vstore8" | "vstore16") {
                    Ok(Ty::Void)
                } else {
                    Ok(self
                        .functions
                        .get(name)
                        .ok_or_else(|| format!("unknown function {name}"))?
                        .ret
                        .clone())
                }
            }
            Expr::Binary { op, lhs, .. } => {
                if is_comparison(*op) {
                    Ok(Ty::Bool)
                } else {
                    self.expr_ty(lhs)
                }
            }
        }
    }
}

fn is_comparison(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
    )
}

fn binary_name(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "add",
        BinaryOp::Sub => "sub",
        BinaryOp::Mul => "mul",
        BinaryOp::Div => "div",
        BinaryOp::Mod => "mod",
        BinaryOp::BitAnd => "and",
        BinaryOp::BitOr => "or",
        BinaryOp::BitXor => "xor",
        BinaryOp::Shl => "shl",
        BinaryOp::Shr => "shr",
        BinaryOp::Eq => "eq",
        BinaryOp::Ne => "ne",
        BinaryOp::Lt => "lt",
        BinaryOp::Le => "le",
        BinaryOp::Gt => "gt",
        BinaryOp::Ge => "ge",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parser, semantic};

    #[test]
    fn ir_has_no_target_registers() {
        let program =
            parser::parse("fn main()->u16{u16 a[4];u16 i=1;a[i]=7;return a[i];}").unwrap();
        semantic::validate(&program).unwrap();
        let ir = emit_ir(&program).unwrap();
        assert!(ir.contains("index %"));
        assert!(ir.contains("storemem.u16"));
        assert!(!ir.contains("R0"));
        assert!(!ir.contains("ALU."));
    }

    #[test]
    fn loops_become_labels_and_jumps() {
        let program = parser::parse("fn main()->u16{u16 i=0;while(i<3){i=i+1;}return i;}").unwrap();
        semantic::validate(&program).unwrap();
        let ir = emit_ir(&program).unwrap();
        assert!(ir.contains("while_test_"));
        assert!(ir.contains("jz %"));
        assert!(ir.contains("jmp while_test_"));
    }

    #[test]
    fn mmio_builtin_becomes_volatile_memory_ir() {
        let program =
            parser::parse("fn main()->u16{vstore8(0xff00,65);vload8(0xff01);return 0;}").unwrap();
        semantic::validate(&program).unwrap();
        let ir = emit_ir(&program).unwrap();
        assert!(ir.contains("storememv.u8"));
        assert!(ir.contains("loadmemv.u8"));
    }

    #[test]
    fn signed_comparison_keeps_operand_type_in_ir() {
        let program = parser::parse(
            "fn less(i16 a,i16 b)->bool{return a<b;} fn main()->u16{less(-1,1);return 0;}",
        )
        .unwrap();
        semantic::validate(&program).unwrap();
        let ir = emit_ir(&program).unwrap();
        assert!(ir.contains("lt.i16"));
        assert!(!ir.contains("lt.u16"));
    }
}
