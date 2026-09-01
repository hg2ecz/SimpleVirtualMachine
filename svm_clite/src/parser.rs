use crate::ast::*;
use crate::lexer::{Tok, lex};

pub fn parse(src: &str) -> Result<Program, String> {
    P { t: lex(src)?, i: 0 }.program()
}

struct P {
    t: Vec<Tok>,
    i: usize,
}

impl P {
    fn peek(&self) -> &Tok {
        &self.t[self.i]
    }

    fn bump(&mut self) -> Tok {
        let token = self.t[self.i].clone();
        self.i += 1;
        token
    }

    fn sym(&mut self, symbol: &str) -> bool {
        if matches!(self.peek(), Tok::Sym(x) if x == symbol) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn need(&mut self, symbol: &str) -> Result<(), String> {
        if self.sym(symbol) {
            Ok(())
        } else {
            Err(format!("expected '{symbol}', got {:?}", self.peek()))
        }
    }

    fn kw(&mut self, keyword: &str) -> bool {
        if matches!(self.peek(), Tok::Id(x) if x == keyword) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn id(&mut self) -> Result<String, String> {
        match self.bump() {
            Tok::Id(name) => Ok(name),
            token => Err(format!("expected identifier, got {token:?}")),
        }
    }

    fn base_ty(&mut self) -> Result<Ty, String> {
        let name = self.id()?;
        match name.as_str() {
            "bool" => Ok(Ty::Bool),
            "i8" => Ok(Ty::I8),
            "u8" => Ok(Ty::U8),
            "i16" => Ok(Ty::I16),
            "u16" => Ok(Ty::U16),
            "void" => Ok(Ty::Void),
            _ => Err(format!("unknown type {name}")),
        }
    }

    fn ptr_ty(&mut self) -> Result<Ty, String> {
        let ty = self.base_ty()?;
        if !self.sym("*") {
            return Ok(ty);
        }
        if ty == Ty::Void {
            return Err("void* is not part of C-Lite".into());
        }
        if self.sym("*") {
            return Err("pointer-to-pointer is not part of C-Lite".into());
        }
        Ok(Ty::Ptr(Box::new(ty)))
    }

    fn declarator(&mut self) -> Result<(String, Ty), String> {
        let mut ty = self.ptr_ty()?;
        let name = self.id()?;

        if self.sym("[") {
            if matches!(ty, Ty::Ptr(_) | Ty::Void) {
                return Err("array element type must be bool/i8/u8/i16/u16".into());
            }
            let length = match self.bump() {
                Tok::Num(n) => n,
                token => return Err(format!("array length expected, got {token:?}")),
            };
            self.need("]")?;
            ty = Ty::Array(Box::new(ty), length);
        }

        Ok((name, ty))
    }

    fn program(mut self) -> Result<Program, String> {
        let mut globals = Vec::new();
        let mut functions = Vec::new();

        while !matches!(self.peek(), Tok::Eof) {
            if matches!(self.peek(), Tok::Id(x) if x == "fn") {
                functions.push(self.function()?);
                continue;
            }

            let (name, ty) = self.declarator()?;
            let init = if self.sym("=") {
                Some(self.expr(0)?)
            } else {
                None
            };
            self.need(";")?;
            globals.push(Global { name, ty, init });
        }

        Ok(Program { globals, functions })
    }

    fn function(&mut self) -> Result<Function, String> {
        if !self.kw("fn") {
            return Err(format!("expected 'fn', got {:?}", self.peek()));
        }

        let name = self.id()?;
        self.need("(")?;
        let mut params = Vec::new();

        if !self.sym(")") {
            loop {
                let (name, ty) = self.declarator()?;
                if matches!(ty, Ty::Array(_, _)) {
                    return Err("array parameters are not allowed; use T*".into());
                }
                params.push(Param { name, ty });

                if self.sym(")") {
                    break;
                }
                self.need(",")?;
            }
        }

        let ret = if self.sym("->") {
            self.ptr_ty()?
        } else {
            Ty::Void
        };
        let body = self.block()?;

        Ok(Function {
            name,
            params,
            ret,
            body,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.need("{")?;
        let mut body = Vec::new();
        while !self.sym("}") {
            body.push(self.stmt()?);
        }
        Ok(body)
    }

    fn stmt(&mut self) -> Result<Stmt, String> {
        if matches!(self.peek(), Tok::Id(x) if matches!(x.as_str(), "bool" | "i8" | "u8" | "i16" | "u16" | "void"))
        {
            let statement = self.simple_stmt()?;
            self.need(";")?;
            return Ok(statement);
        }

        if self.kw("return") {
            if self.sym(";") {
                return Ok(Stmt::Return(None));
            }
            let expr = self.expr(0)?;
            self.need(";")?;
            return Ok(Stmt::Return(Some(expr)));
        }

        if self.kw("break") {
            self.need(";")?;
            return Ok(Stmt::Break);
        }

        if self.kw("continue") {
            self.need(";")?;
            return Ok(Stmt::Continue);
        }

        if self.kw("while") {
            self.need("(")?;
            let cond = self.expr(0)?;
            self.need(")")?;
            return Ok(Stmt::While {
                cond,
                body: self.block()?,
            });
        }

        if self.kw("if") {
            self.need("(")?;
            let cond = self.expr(0)?;
            self.need(")")?;
            let then_body = self.block()?;
            let else_body = if self.kw("else") {
                if matches!(self.peek(), Tok::Id(x) if x == "if") {
                    vec![self.stmt()?]
                } else {
                    self.block()?
                }
            } else {
                Vec::new()
            };
            return Ok(Stmt::If {
                cond,
                then_body,
                else_body,
            });
        }

        let statement = self.simple_stmt()?;
        self.need(";")?;
        Ok(statement)
    }

    fn simple_stmt(&mut self) -> Result<Stmt, String> {
        if matches!(self.peek(), Tok::Id(x) if matches!(x.as_str(), "bool" | "i8" | "u8" | "i16" | "u16" | "void"))
        {
            let (name, ty) = self.declarator()?;
            let init = if self.sym("=") {
                Some(self.expr(0)?)
            } else {
                None
            };
            return Ok(Stmt::Let { name, ty, init });
        }

        let lhs = self.expr(0)?;
        if self.sym("=") {
            Ok(Stmt::Assign {
                lhs,
                rhs: self.expr(0)?,
            })
        } else {
            Ok(Stmt::Expr(lhs))
        }
    }

    fn expr(&mut self, min_precedence: u8) -> Result<Expr, String> {
        let mut lhs = if self.sym("-") {
            Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(self.expr(11)?),
            }
        } else if self.sym("~") {
            Expr::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(self.expr(11)?),
            }
        } else if self.sym("&") {
            Expr::Unary {
                op: UnaryOp::AddrOf,
                expr: Box::new(self.expr(11)?),
            }
        } else if self.sym("*") {
            Expr::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(self.expr(11)?),
            }
        } else {
            self.primary()?
        };

        loop {
            let Some((precedence, op)) = self.binop() else {
                break;
            };
            if precedence < min_precedence {
                break;
            }
            self.bump();
            let rhs = self.expr(precedence + 1)?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }

        Ok(lhs)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let mut expr = match self.bump() {
            Tok::Num(n) => Expr::Int(n),
            Tok::Id(name) if name == "true" => Expr::Bool(true),
            Tok::Id(name) if name == "false" => Expr::Bool(false),
            Tok::Id(name) => {
                if self.sym("(") {
                    let mut args = Vec::new();
                    if !self.sym(")") {
                        loop {
                            args.push(self.expr(0)?);
                            if self.sym(")") {
                                break;
                            }
                            self.need(",")?;
                        }
                    }
                    Expr::Call { name, args }
                } else {
                    Expr::Var(name)
                }
            }
            Tok::Sym(symbol) if symbol == "(" => {
                let expr = self.expr(0)?;
                self.need(")")?;
                expr
            }
            token => return Err(format!("expected expression, got {token:?}")),
        };

        while self.sym("[") {
            let index = self.expr(0)?;
            self.need("]")?;
            expr = Expr::Index {
                base: Box::new(expr),
                index: Box::new(index),
            };
        }

        Ok(expr)
    }

    fn binop(&self) -> Option<(u8, BinaryOp)> {
        let Tok::Sym(symbol) = self.peek() else {
            return None;
        };
        Some(match symbol.as_str() {
            "|" => (1, BinaryOp::BitOr),
            "^" => (2, BinaryOp::BitXor),
            "&" => (3, BinaryOp::BitAnd),
            "==" => (4, BinaryOp::Eq),
            "!=" => (4, BinaryOp::Ne),
            "<" => (5, BinaryOp::Lt),
            "<=" => (5, BinaryOp::Le),
            ">" => (5, BinaryOp::Gt),
            ">=" => (5, BinaryOp::Ge),
            "<<" => (6, BinaryOp::Shl),
            ">>" => (6, BinaryOp::Shr),
            "+" => (7, BinaryOp::Add),
            "-" => (7, BinaryOp::Sub),
            "*" => (8, BinaryOp::Mul),
            "/" => (8, BinaryOp::Div),
            "%" => (8, BinaryOp::Mod),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_pointer_and_loop() {
        let source = "fn sum(u16* p, u16 n) -> u16 { u16 i=0; u16 a[4]; while(i<n){ a[i]=p[i]; i=i+1; } return a[0]; }";
        let program = parse(source).unwrap();
        assert_eq!(program.functions.len(), 1);
        assert!(matches!(program.functions[0].params[0].ty, Ty::Ptr(_)));
    }

    #[test]
    fn parses_c_like_globals_and_arrays() {
        let program = parse("u16 counter=1; u8 data[16]; fn main()->u16{return counter;}").unwrap();
        assert_eq!(program.globals.len(), 2);
        assert!(matches!(program.globals[1].ty, Ty::Array(_, 16)));
    }

    #[test]
    fn pointer_to_pointer_is_intentionally_rejected() {
        let error = parse("fn f(u16** p)->u16{return 0;} fn main()->u16{return 0;}").unwrap_err();
        assert!(error.contains("pointer-to-pointer"));
    }

    #[test]
    fn parses_bool_type_and_literals() {
        let program = parse("bool ready=true; fn less(u16 a,u16 b)->bool{return a<b;} fn main()->u16{bool x=false; if(less(1,2)){x=true;} if(x){return 1;} return 0;}").unwrap();
        assert!(matches!(program.globals[0].ty, Ty::Bool));
        assert!(matches!(program.functions[0].ret, Ty::Bool));
    }
}
