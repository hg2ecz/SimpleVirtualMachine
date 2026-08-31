use crate::model::*;
#[cfg(test)]
use crate::optimized::pipeline::compile_source;

pub(crate) fn parse_source(source: &str) -> Result<Program, String> {
    let tokens = Lexer::new(source).lex()?;
    Parser::new(tokens).parse_program()
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Id(String),
    Num(u16),
    Str(String),
    Sym(String),
    Eof,
}
struct Lexer<'a> {
    s: &'a [u8],
    i: usize,
}
impl<'a> Lexer<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            s: s.as_bytes(),
            i: 0,
        }
    }
    fn lex(mut self) -> Result<Vec<Tok>, String> {
        let mut v = Vec::new();
        loop {
            self.ws();
            if self.i >= self.s.len() {
                v.push(Tok::Eof);
                break;
            }
            let c = self.s[self.i];
            if c.is_ascii_alphabetic() || c == b'_' {
                let st = self.i;
                self.i += 1;
                while self.i < self.s.len()
                    && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_')
                {
                    self.i += 1
                }
                v.push(Tok::Id(String::from_utf8_lossy(&self.s[st..self.i]).into()));
                continue;
            }
            if c == b'"' {
                self.i += 1;
                let mut out = String::new();
                while self.i < self.s.len() && self.s[self.i] != b'"' {
                    let ch = self.s[self.i];
                    self.i += 1;
                    if ch == b'\\' {
                        if self.i >= self.s.len() {
                            return Err("unterminated string escape".into());
                        }
                        let e = self.s[self.i];
                        self.i += 1;
                        out.push(match e {
                            b'n' => '\n',
                            b'r' => '\r',
                            b't' => '\t',
                            b'0' => '\0',
                            b'\\' => '\\',
                            b'"' => '"',
                            _ => {
                                return Err(format!("unsupported string escape \\\\{}", e as char));
                            }
                        });
                    } else {
                        if ch < 0x20 && ch != b'\t' {
                            return Err("control character in string literal".into());
                        }
                        out.push(ch as char);
                    }
                }
                if self.i >= self.s.len() {
                    return Err("unterminated string literal".into());
                }
                self.i += 1;
                v.push(Tok::Str(out));
                continue;
            }
            if c.is_ascii_digit() {
                let st = self.i;
                self.i += 1;
                while self.i < self.s.len()
                    && (self.s[self.i].is_ascii_hexdigit() || matches!(self.s[self.i], b'x' | b'X'))
                {
                    self.i += 1
                }
                let t = String::from_utf8_lossy(&self.s[st..self.i]);
                let n = if t.starts_with("0x") || t.starts_with("0X") {
                    u16::from_str_radix(&t[2..], 16)
                } else {
                    t.parse()
                }
                .map_err(|_| format!("bad number {t}"))?;
                v.push(Tok::Num(n));
                continue;
            }
            if self.i + 2 < self.s.len() {
                let t = String::from_utf8_lossy(&self.s[self.i..self.i + 3]).to_string();
                if ["<<=", ">>="].contains(&t.as_str()) {
                    self.i += 3;
                    v.push(Tok::Sym(t));
                    continue;
                }
            }
            if self.i + 1 < self.s.len() {
                let t = String::from_utf8_lossy(&self.s[self.i..self.i + 2]).to_string();
                if [
                    "==", "!=", "<=", ">=", "<<", ">>", "++", "--", "+=", "-=", "*=", "/=", "%=",
                    "&=", "|=", "^=", "&&", "||",
                ]
                .contains(&t.as_str())
                {
                    self.i += 2;
                    v.push(Tok::Sym(t));
                    continue;
                }
            }
            if b"{}[]();,+-*/%&|^~!<>=\"".contains(&c) {
                self.i += 1;
                v.push(Tok::Sym((c as char).to_string()));
                continue;
            }
            return Err(format!("unexpected character '{}'", c as char));
        }
        Ok(v)
    }
    fn ws(&mut self) {
        loop {
            while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
                self.i += 1
            }
            if self.i + 1 < self.s.len() && self.s[self.i] == b'/' && self.s[self.i + 1] == b'/' {
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.i += 1
                }
                continue;
            }
            if self.i + 1 < self.s.len() && self.s[self.i] == b'/' && self.s[self.i + 1] == b'*' {
                self.i += 2;
                while self.i + 1 < self.s.len()
                    && !(self.s[self.i] == b'*' && self.s[self.i + 1] == b'/')
                {
                    self.i += 1
                }
                self.i = (self.i + 2).min(self.s.len());
                continue;
            }
            break;
        }
    }
}

struct Parser {
    t: Vec<Tok>,
    i: usize,
    loop_depth: usize,
}
impl Parser {
    fn new(t: Vec<Tok>) -> Self {
        Self {
            t,
            i: 0,
            loop_depth: 0,
        }
    }
    fn peek(&self) -> &Tok {
        &self.t[self.i]
    }
    fn bump(&mut self) -> Tok {
        let x = self.t[self.i].clone();
        self.i += 1;
        x
    }
    fn sym(&mut self, s: &str) -> bool {
        if self.peek() == &Tok::Sym(s.into()) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn need_sym(&mut self, s: &str) -> Result<(), String> {
        if self.sym(s) {
            Ok(())
        } else {
            Err(format!("expected '{s}'"))
        }
    }
    fn id(&mut self) -> Result<String, String> {
        match self.bump() {
            Tok::Id(s) => Ok(s),
            x => Err(format!("expected identifier, got {x:?}")),
        }
    }
    fn ty(&mut self) -> Result<Ty, String> {
        match self.bump() {
            Tok::Id(s) if s == "bool" => Ok(Ty::Bool),
            Tok::Id(s) if s == "i8" => Ok(Ty::I8),
            Tok::Id(s) if s == "u8" => Ok(Ty::U8),
            Tok::Id(s) if s == "i16" || s == "int" => Ok(Ty::I16),
            Tok::Id(s) if s == "u16" => Ok(Ty::U16),
            Tok::Id(s) if s == "i32" || s == "long" => Ok(Ty::I32),
            Tok::Id(s) if s == "u32" => Ok(Ty::U32),
            Tok::Id(s) if s == "i64" => Ok(Ty::I64),
            Tok::Id(s) if s == "u64" => Ok(Ty::U64),
            Tok::Id(s) if s == "void" => Ok(Ty::Void),
            x => Err(format!("expected type, got {x:?}")),
        }
    }
    fn is_ty(&self) -> bool {
        matches!(self.peek(),Tok::Id(s) if matches!(s.as_str(), "bool"|"i8"|"u8"|"i16"|"u16"|"int"|"i32"|"u32"|"long"|"i64"|"u64"|"void"))
    }
    fn array_len(&mut self) -> Result<u16, String> {
        if !self.sym("[") {
            return Ok(1);
        }
        let n = match self.bump() {
            Tok::Num(n) if n > 0 => n,
            _ => return Err("array size must be a positive numeric constant".into()),
        };
        self.need_sym("]")?;
        Ok(n)
    }
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut globals = vec![];
        let mut functions = vec![];
        while !matches!(self.peek(), Tok::Eof) {
            let ty = self.ty()?;
            let name = self.id()?;
            if self.sym("(") {
                let mut params = vec![];
                if !self.sym(")") {
                    loop {
                        let pt = self.ty()?;
                        let pn = self.id()?;
                        if self.sym("[") {
                            return Err(
                                "array parameters are not supported in subset+ phase 1".into()
                            );
                        }
                        params.push(VarDecl {
                            ty: pt,
                            name: pn,
                            len: 1,
                            init: None,
                        });
                        if self.sym(")") {
                            break;
                        }
                        self.need_sym(",")?
                    }
                }
                let body = self.stmt()?;
                functions.push(Function {
                    ret: ty,
                    name,
                    params,
                    body,
                })
            } else {
                if ty == Ty::Void {
                    return Err("void global is invalid".into());
                }
                let len = self.array_len()?;
                let init = if self.sym("=") {
                    if len != 1 {
                        return Err(
                            "array initializers are not supported in subset+ phase 1".into()
                        );
                    }
                    Some(self.expr()?)
                } else {
                    None
                };
                self.need_sym(";")?;
                globals.push(VarDecl {
                    ty,
                    name,
                    len,
                    init,
                })
            }
        }
        Ok(Program { globals, functions })
    }
    fn stmt(&mut self) -> Result<Stmt, String> {
        if self.sym("{") {
            let mut v = vec![];
            while !self.sym("}") {
                v.push(self.stmt()?)
            }
            return Ok(Stmt::Block(v));
        }
        if self.sym(";") {
            return Ok(Stmt::Empty);
        }
        if self.is_ty() {
            let ty = self.ty()?;
            if ty == Ty::Void {
                return Err("void local is invalid".into());
            }
            let name = self.id()?;
            let len = self.array_len()?;
            let init = if self.sym("=") {
                if len != 1 {
                    return Err("array initializers are not supported in subset+ phase 1".into());
                }
                Some(self.expr()?)
            } else {
                None
            };
            self.need_sym(";")?;
            return Ok(Stmt::Var(VarDecl {
                ty,
                name,
                len,
                init,
            }));
        }
        if let Tok::Id(k) = self.peek().clone() {
            match k.as_str() {
                "if" => {
                    self.bump();
                    self.need_sym("(")?;
                    let c = self.expr()?;
                    self.need_sym(")")?;
                    let a = Box::new(self.stmt()?);
                    let b = if matches!(self.peek(),Tok::Id(s) if s=="else") {
                        self.bump();
                        Some(Box::new(self.stmt()?))
                    } else {
                        None
                    };
                    return Ok(Stmt::If(c, a, b));
                }
                "while" => {
                    self.bump();
                    self.need_sym("(")?;
                    let c = self.expr()?;
                    self.need_sym(")")?;
                    self.loop_depth += 1;
                    let body = self.stmt();
                    self.loop_depth -= 1;
                    return Ok(Stmt::While(c, Box::new(body?)));
                }
                "do" => {
                    self.bump();
                    self.loop_depth += 1;
                    let body = self.stmt();
                    self.loop_depth -= 1;
                    let body = body?;
                    match self.bump() {
                        Tok::Id(s) if s == "while" => {}
                        _ => return Err("expected 'while' after do body".into()),
                    }
                    self.need_sym("(")?;
                    let c = self.expr()?;
                    self.need_sym(")")?;
                    self.need_sym(";")?;
                    return Ok(Stmt::DoWhile(Box::new(body), c));
                }
                "break" => {
                    if self.loop_depth == 0 {
                        return Err("break is only valid inside a loop".into());
                    }
                    self.bump();
                    self.need_sym(";")?;
                    return Ok(Stmt::Break);
                }
                "continue" => {
                    if self.loop_depth == 0 {
                        return Err("continue is only valid inside a loop".into());
                    }
                    self.bump();
                    self.need_sym(";")?;
                    return Ok(Stmt::Continue);
                }
                "for" => {
                    self.bump();
                    self.need_sym("(")?;
                    let init = if self.sym(";") {
                        None
                    } else {
                        let s = self.simple_stmt_no_semi()?;
                        self.need_sym(";")?;
                        Some(Box::new(s))
                    };
                    let cond = if self.sym(";") {
                        None
                    } else {
                        let e = self.expr()?;
                        self.need_sym(";")?;
                        Some(e)
                    };
                    let step = if self.sym(")") {
                        None
                    } else {
                        let s = self.simple_stmt_no_semi()?;
                        self.need_sym(")")?;
                        Some(Box::new(s))
                    };
                    self.loop_depth += 1;
                    let body = self.stmt();
                    self.loop_depth -= 1;
                    return Ok(Stmt::For(init, cond, step, Box::new(body?)));
                }
                "return" => {
                    self.bump();
                    if self.sym(";") {
                        return Ok(Stmt::Return(None));
                    }
                    let e = self.expr()?;
                    self.need_sym(";")?;
                    return Ok(Stmt::Return(Some(e)));
                }
                _ => {}
            }
        }
        let s = self.simple_stmt_no_semi()?;
        self.need_sym(";")?;
        Ok(s)
    }
    fn simple_stmt_no_semi(&mut self) -> Result<Stmt, String> {
        if self.is_ty() {
            let ty = self.ty()?;
            if ty == Ty::Void {
                return Err("void local is invalid".into());
            }
            let name = self.id()?;
            let len = self.array_len()?;
            let init = if self.sym("=") {
                if len != 1 {
                    return Err("array initializers are not supported in subset+ phase 1".into());
                }
                Some(self.expr()?)
            } else {
                None
            };
            return Ok(Stmt::Var(VarDecl {
                ty,
                name,
                len,
                init,
            }));
        }
        // Subset+ phase 1: statement-level ++/-- and compound assignment,
        // including fixed-size array elements.
        if let Tok::Id(name) = self.peek().clone() {
            let save = self.i;
            self.bump();
            let idx = if self.sym("[") {
                let x = self.expr()?;
                self.need_sym("]")?;
                Some(x)
            } else {
                None
            };
            let op = match self.peek() {
                Tok::Sym(x) => Some(x.clone()),
                _ => None,
            };
            if let Some(op) = op {
                let bop = match op.as_str() {
                    "+=" => Some(BinOp::Add),
                    "-=" => Some(BinOp::Sub),
                    "*=" => Some(BinOp::Mul),
                    "/=" => Some(BinOp::Div),
                    "%=" => Some(BinOp::Mod),
                    "&=" => Some(BinOp::And),
                    "|=" => Some(BinOp::Or),
                    "^=" => Some(BinOp::Xor),
                    "<<=" => Some(BinOp::Shl),
                    ">>=" => Some(BinOp::Shr),
                    _ => None,
                };
                if op == "=" || op == "++" || op == "--" || bop.is_some() {
                    self.bump();
                    if let Some(i) = idx {
                        if op != "=" && crate::semantic::expr_has_call(&i) {
                            return Err(
                                "compound array update requires a side-effect-free index".into()
                            );
                        }
                        let rhs = if op == "=" {
                            self.expr()?
                        } else if op == "++" {
                            Expr::Binary(
                                BinOp::Add,
                                Box::new(Expr::Call(format!("__index__{}", name), vec![i.clone()])),
                                Box::new(Expr::Num(1)),
                            )
                        } else if op == "--" {
                            Expr::Binary(
                                BinOp::Sub,
                                Box::new(Expr::Call(format!("__index__{}", name), vec![i.clone()])),
                                Box::new(Expr::Num(1)),
                            )
                        } else {
                            Expr::Binary(
                                bop.unwrap(),
                                Box::new(Expr::Call(format!("__index__{}", name), vec![i.clone()])),
                                Box::new(self.expr()?),
                            )
                        };
                        return Ok(Stmt::Expr(Expr::Call(
                            format!("__store__{}", name),
                            vec![i, rhs],
                        )));
                    }
                    let old = Expr::Var(name.clone());
                    let rhs = if op == "=" {
                        self.expr()?
                    } else if op == "++" {
                        Expr::Binary(BinOp::Add, Box::new(old), Box::new(Expr::Num(1)))
                    } else if op == "--" {
                        Expr::Binary(BinOp::Sub, Box::new(old), Box::new(Expr::Num(1)))
                    } else {
                        Expr::Binary(bop.unwrap(), Box::new(old), Box::new(self.expr()?))
                    };
                    return Ok(Stmt::Assign(name, rhs));
                }
            }
            self.i = save;
        }
        Ok(Stmt::Expr(self.expr()?))
    }
    fn expr(&mut self) -> Result<Expr, String> {
        self.bin(0)
    }
    fn bin(&mut self, min: u8) -> Result<Expr, String> {
        let mut lhs = self.unary()?;
        loop {
            let (op, p) = match self.peek() {
                Tok::Sym(s) => match s.as_str() {
                    "||" => (BinOp::LogicalOr, 0),
                    "&&" => (BinOp::LogicalAnd, 1),
                    "|" => (BinOp::Or, 2),
                    "^" => (BinOp::Xor, 3),
                    "&" => (BinOp::And, 4),
                    "==" => (BinOp::Eq, 5),
                    "!=" => (BinOp::Ne, 5),
                    "<" => (BinOp::Lt, 6),
                    ">" => (BinOp::Gt, 6),
                    "<=" => (BinOp::Le, 6),
                    ">=" => (BinOp::Ge, 6),
                    "<<" => (BinOp::Shl, 7),
                    ">>" => (BinOp::Shr, 7),
                    "+" => (BinOp::Add, 8),
                    "-" => (BinOp::Sub, 8),
                    "*" => (BinOp::Mul, 9),
                    "/" => (BinOp::Div, 9),
                    "%" => (BinOp::Mod, 9),
                    _ => break,
                },
                _ => break,
            };
            if p < min {
                break;
            }
            self.bump();
            let rhs = self.bin(p + 1)?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs))
        }
        Ok(lhs)
    }
    fn unary(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Tok::Id(s) if s == "sizeof") {
            self.bump();
            self.need_sym("(")?;
            let e = match self.bump() {
                Tok::Id(s) if matches!(s.as_str(), "bool" | "i8" | "u8") => Expr::Num(1),
                Tok::Id(s) if matches!(s.as_str(), "i16" | "u16" | "int") => Expr::Num(2),
                Tok::Id(s) if matches!(s.as_str(), "i32" | "u32" | "long") => Expr::Num(4),
                Tok::Id(s) if matches!(s.as_str(), "i64" | "u64") => Expr::Num(8),
                Tok::Id(s) => Expr::SizeOfName(s),
                x => return Err(format!("sizeof expects a type or object name, got {x:?}")),
            };
            self.need_sym(")")?;
            return Ok(e);
        }
        if self.sym("&") {
            return match self.bump() {
                Tok::Id(s) => Ok(Expr::AddrOf(s)),
                x => Err(format!("address-of expects an object name, got {x:?}")),
            };
        }
        if self.sym("-") {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
        }
        if self.sym("~") {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.unary()?)));
        }
        if self.sym("!") {
            return Ok(Expr::Unary(UnOp::LogicalNot, Box::new(self.unary()?)));
        }
        self.primary()
    }
    fn primary(&mut self) -> Result<Expr, String> {
        match self.bump() {
            Tok::Num(n) => Ok(Expr::Num(n)),
            Tok::Str(x) => Ok(Expr::Str(x)),
            Tok::Id(s) => {
                if self.sym("(") {
                    let mut a = vec![];
                    if !self.sym(")") {
                        loop {
                            a.push(self.expr()?);
                            if self.sym(")") {
                                break;
                            }
                            self.need_sym(",")?
                        }
                    }
                    Ok(Expr::Call(s, a))
                } else if self.sym("[") {
                    let idx = self.expr()?;
                    self.need_sym("]")?;
                    Ok(Expr::Call(format!("__index__{}", s), vec![idx]))
                } else {
                    Ok(Expr::Var(s))
                }
            }
            Tok::Sym(s) if s == "(" => {
                let e = self.expr()?;
                self.need_sym(")")?;
                Ok(e)
            }
            x => Err(format!("expected expression, got {x:?}")),
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    use super::*;

    fn reg(src: &str, opt: OptLevel) -> String {
        compile_source(src, Target::Register, opt).expect("compile")
    }

    #[test]
    fn o1_uses_native_inc() {
        let src = "u16 main(){ u16 x=2; x=x+1; return x; }";
        let asm = reg(src, OptLevel::O1);
        assert!(asm.contains("INC R0"));
    }

    #[test]
    fn o2_propagates_constants() {
        let src = "u16 main(){ u16 a=5; u16 b=a+1; return b; }";
        let asm = reg(src, OptLevel::O2);
        assert!(asm.contains("MOVI R0, 6"));
    }

    #[test]
    fn o0_does_not_run_ast_strength_reduction() {
        let src = "u16 main(){ u16 x=2; x=x+1; return x; }";
        let asm = reg(src, OptLevel::O0);
        assert!(!asm.contains("; Optimization: O1"));
        assert!(asm.contains("Optimization: O0"));
    }

    #[test]
    fn register_backend_uses_immediate_alu_forms() {
        let src = "u16 main(){ u16 x; x=7; x=x+5; return x==12; }";
        let asm = reg(src, OptLevel::O0);
        assert!(asm.contains("ADDI R0, 5"));
        assert!(asm.contains("CMPI R0, 12"));
    }

    #[test]
    fn register_commutative_result_stays_in_r0() {
        let src = "u16 main(){ u16 a; u16 b; a=3; b=4; return a+b; }";
        let asm = reg(src, OptLevel::O0);
        assert!(asm.contains("ADD R0, R1"));
        assert!(!asm.contains("ADD R1, R0\n    MOV R0, R1"));
    }

    #[test]
    fn optimized_build_drops_unreachable_functions_transitively() {
        let src = "u16 dead2(){ u16 pad[32]; return 3; } u16 dead1(){ return dead2(); } u16 leaf(){ return 7; } u16 helper(){ return leaf(); } u16 main(){ return helper(); }";
        let o0 = reg(src, OptLevel::O0);
        assert!(o0.contains("dead1:"));
        assert!(o0.contains("dead2:"));
        for level in [OptLevel::O1, OptLevel::O2, OptLevel::Os] {
            let asm = reg(src, level);
            assert!(!asm.contains("dead1:"));
            assert!(!asm.contains("dead2:"));
            assert!(asm.contains("helper:"));
            assert!(asm.contains("leaf:"));
        }
    }

    #[test]
    fn memreg_backend_uses_immediate_alu_forms() {
        // AND is the preferred hot logical operation in the current MemReg ISA.
        // Keep this regression focused on direct immediate lowering rather than
        // routing the literal through the expression stack/scratch cell.
        let src = "u16 main(){ u16 x; x=7; return (x&0x55)==5; }";
        let asm = compile_source(src, Target::MemReg, OptLevel::O0).expect("compile");
        assert!(asm.contains("ANDI 85"));
        assert!(asm.contains("CMPI 5"));
        assert!(!asm.contains("PUSHW\n    LDI 85"));
    }
}

#[cfg(test)]
mod semantic_review_tests {
    use super::*;

    #[test]
    fn rejects_unknown_variable() {
        let src = "u16 main(){ x = 1; return 0; }";
        assert!(
            compile_source(src, Target::Register, OptLevel::O1)
                .unwrap_err()
                .contains("unknown variable")
        );
    }

    #[test]
    fn rejects_bad_builtin_arity() {
        let src = "u16 main(){ return load8(1,2); }";
        assert!(
            compile_source(src, Target::Register, OptLevel::O1)
                .unwrap_err()
                .contains("expects 1 argument")
        );
    }

    #[test]
    fn rejects_void_call_as_value() {
        let src = "u16 main(){ return store8(0x10,1); }";
        assert!(
            compile_source(src, Target::Register, OptLevel::O1)
                .unwrap_err()
                .contains("void")
        );
    }

    #[test]
    fn optimizer_does_not_hide_invalid_dead_code() {
        let src = "u16 main(){ if(0){ missing(); } return 0; }";
        assert!(
            compile_source(src, Target::Register, OptLevel::O2)
                .unwrap_err()
                .contains("unknown function")
        );
    }

    #[test]
    fn supports_more_than_four_scalar_parameters_on_all_targets() {
        let src = "u16 f(u16 a,u16 b,u16 c,u16 d,u16 e,u16 f,u16 g){ return a+b+c+d+e+f+g; } u16 main(){ return f(1,2,3,4,5,6,7); }";
        for target in [
            Target::Register,
            Target::Stack,
            Target::Accumulator,
            Target::MemReg,
            Target::LoadStore,
            Target::RegMem,
            Target::Memory2Memory,
            Target::Belt,
            Target::Tta,
        ] {
            assert!(
                compile_source(src, target, OptLevel::O0).is_ok(),
                "target {target:?}"
            );
        }
    }

    #[test]
    fn register_argument_staging_uses_stack_before_callee_slots() {
        let src = "u16 id(u16 x){return x;} u16 f(u16 a,u16 b,u16 c,u16 d,u16 e){return a+b+c+d+e;} u16 main(){return f(1,id(2),3,4,5);}";
        let asm = compile_source(src, Target::Register, OptLevel::O0).unwrap();
        let call = asm.find("CALL f").expect("CALL f");
        let prefix = &asm[..call];
        assert!(prefix.matches("PUSH R0").count() >= 5);
        assert!(prefix.matches("POP R0").count() >= 5);
    }
}

#[cfg(test)]
mod loop_optimizer_regression_tests {
    use super::*;

    #[test]
    fn o2_does_not_freeze_while_induction_variable() {
        let src = r#"
            u16 main() {
                u16 i;
                i = 0;
                while (i < 16) { i = i + 1; }
                return i;
            }
        "#;
        let asm = compile_source(src, Target::Accumulator, OptLevel::O2).unwrap();
        // The condition must still load i inside the loop. If propagation froze
        // i=0 into the condition, the generated loop would be unconditional.
        assert!(asm.contains("while"));
        assert!(asm.contains("CMPX") || asm.contains("CMPI"));
        assert!(asm.contains("INC"));
    }

    #[test]
    fn os_does_not_freeze_for_condition() {
        let src = r#"
            u16 main() {
                u16 i;
                for (i = 0; i < 16; i = i + 1) { }
                return i;
            }
        "#;
        let asm = compile_source(src, Target::Accumulator, OptLevel::Os).unwrap();
        assert!(asm.contains("for"));
        assert!(asm.contains("CMPX") || asm.contains("CMPI"));
        assert!(asm.contains("INC"));
    }
}

#[cfg(test)]
mod subset_plus_phase1_tests {
    use super::*;

    #[test]
    fn fixed_arrays_compile() {
        let src =
            "u8 a[10]; u16 w[4]; u16 main(){ u16 i; i=2; a[i]=7; w[1]=0x1234; return a[i]+w[1]; }";
        assert!(compile_source(src, Target::Register, OptLevel::O2).is_ok());
    }

    #[test]
    fn scalar_compound_and_increment_compile() {
        let src = "u16 main(){ u16 i; i=0; i++; i+=3; i<<=1; i--; return i; }";
        assert!(compile_source(src, Target::Stack, OptLevel::Os).is_ok());
    }

    #[test]
    fn constant_array_oob_is_rejected() {
        let src = "u8 a[4]; u16 main(){ return a[4]; }";
        assert!(
            compile_source(src, Target::Accumulator, OptLevel::O0)
                .unwrap_err()
                .contains("out of bounds")
        );
    }
}
