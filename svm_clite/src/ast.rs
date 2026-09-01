#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    Bool,
    I8,
    U8,
    I16,
    U16,
    Void,
    Ptr(Box<Ty>),
    Array(Box<Ty>, u16),
}

impl Ty {
    pub fn width(&self) -> u16 {
        match self {
            Ty::Bool | Ty::I8 | Ty::U8 => 1,
            Ty::I16 | Ty::U16 | Ty::Ptr(_) => 2,
            Ty::Array(t, n) => t.width() * *n,
            Ty::Void => 0,
        }
    }

    pub fn clir_suffix(&self) -> &'static str {
        match self {
            Ty::Bool => "bool",
            Ty::I8 => "i8",
            Ty::U8 => "u8",
            Ty::I16 => "i16",
            Ty::U16 | Ty::Ptr(_) => "u16",
            Ty::Void => "void",
            Ty::Array(_, _) => "array",
        }
    }

    pub fn name(&self) -> String {
        match self {
            Ty::Bool => "bool".into(),
            Ty::I8 => "i8".into(),
            Ty::U8 => "u8".into(),
            Ty::I16 => "i16".into(),
            Ty::U16 => "u16".into(),
            Ty::Void => "void".into(),
            Ty::Ptr(t) => format!("{}*", t.name()),
            Ty::Array(t, n) => format!("{}[{}]", t.name(), n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub globals: Vec<Global>,
    pub functions: Vec<Function>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Global {
    pub name: String,
    pub ty: Ty,
    pub init: Option<Expr>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Ty,
    pub body: Vec<Stmt>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Ty,
        init: Option<Expr>,
    },
    Assign {
        lhs: Expr,
        rhs: Expr,
    },
    Expr(Expr),
    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Bool(bool),
    Int(u16),
    Var(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    BitNot,
    AddrOf,
    Deref,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
