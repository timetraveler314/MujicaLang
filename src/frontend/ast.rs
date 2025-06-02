use crate::frontend::ty::Ty;

#[derive(Debug)]
pub enum ASTExpr {
    Atom(ASTAtom),
    If {
        cond: Box<ASTExpr>,
        then: Box<ASTExpr>,
        else_: Box<ASTExpr>,
    },
    Let {
        bind: OptionallyTypedIdent,
        value: Box<ASTExpr>,
        body: Box<ASTExpr>,
    },
    /// Single argument function application
    Apply(Box<ASTExpr>, Box<ASTExpr>),
    /// Single argument lambda expression
    Lambda {
        arg: OptionallyTypedIdent,
        body: Box<ASTExpr>,
    },
}

#[derive(Debug)]
pub enum ASTAtom {
    Int(i32),
    Var(String),
    Op(OpType),
}

#[derive(Debug)]
pub enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
}

#[macro_export]
macro_rules! curry_binop {
    ($op:expr, $l:expr, $r:expr) => {
        ASTExpr::Apply(
            Box::new(ASTExpr::Apply(
                Box::new(ASTExpr::Atom(ASTAtom::Op($op))),
                Box::new($l),
            )),
            Box::new($r),
        )
    };
}

#[derive(Debug)]
pub struct OptionallyTypedIdent {
    pub name: String,
    pub ty: Option<Ty>,
}

impl OptionallyTypedIdent {
    pub fn new(name: String, ty: Option<Ty>) -> Self {
        OptionallyTypedIdent { name, ty }
    }
}