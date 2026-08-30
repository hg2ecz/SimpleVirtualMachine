use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Register,
    Stack,
    Accumulator,
    MemReg,
    LoadStore,
    RegMem,
    Memory2Memory,
    Belt,
    Tta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptLevel {
    O0,
    O1,
    O2,
    Os,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    Void,
}

impl Ty {
    pub fn size(self) -> u16 {
        match self {
            Ty::Bool | Ty::I8 | Ty::U8 => 1,
            Ty::I16 | Ty::U16 => 2,
            Ty::I32 | Ty::U32 => 4,
            Ty::I64 | Ty::U64 => 8,
            Ty::Void => 0,
        }
    }
    pub fn is_wide(self) -> bool {
        self.size() > 2
    }
    pub fn is_signed(self) -> bool {
        matches!(self, Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64)
    }
    pub fn is_integer(self) -> bool {
        self != Ty::Void
    }
}

#[derive(Debug, Clone)]
pub struct Program {
    pub globals: Vec<VarDecl>,
    pub functions: Vec<Function>,
}
#[derive(Debug, Clone)]
pub struct VarDecl {
    pub ty: Ty,
    pub name: String,
    /// Number of elements. Scalars use 1; phase-1 subset+ arrays are fixed-size.
    pub len: u16,
    pub init: Option<Expr>,
}
#[derive(Debug, Clone)]
pub struct Function {
    pub ret: Ty,
    pub name: String,
    pub params: Vec<VarDecl>,
    pub body: Stmt,
}
#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Var(VarDecl),
    Assign(String, Expr),
    Expr(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    DoWhile(Box<Stmt>, Expr),
    Break,
    Continue,
    For(
        Option<Box<Stmt>>,
        Option<Expr>,
        Option<Box<Stmt>>,
        Box<Stmt>,
    ),
    Return(Option<Expr>),
    Empty,
}
#[derive(Debug, Clone)]
pub enum Expr {
    Num(u16),
    Var(String),
    Call(String, Vec<Expr>),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Str(String),
    SizeOfName(String),
    /// Address of a named object; lowered to a 16-bit absolute address before codegen.
    AddrOf(String),
}
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
    LogicalNot,
    Inc1,
    Dec1,
    Shl1,
    Shr1,
}
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    LogicalOr,
    LogicalAnd,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ty: Ty,
    pub addr: u16,
    pub len: u16,
}
#[derive(Debug, Clone)]
pub struct Layout {
    pub globals: HashMap<String, VarInfo>,
    pub locals: HashMap<(String, String), VarInfo>,
    pub funcs: HashMap<String, FunctionSig>,
}
#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub ret: Ty,
    pub params: Vec<Ty>,
}
