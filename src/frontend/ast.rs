use crate::core::common::{OpType};
use crate::core::ty::{Type};

#[derive(Debug)]
pub enum ASTExpr {
    Atom(ASTAtom),
    Op {
        op: OpType,
        args: Vec<ASTExpr>,
    },
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
    LetFun {
        bind: OptionallyTypedIdent,
        args: Vec<OptionallyTypedIdent>,
        body: Box<ASTExpr>,
        body2: Box<ASTExpr>,
    },
    Call {
        closure: Box<ASTExpr>,
        args: Vec<ASTExpr>,
    },
}

#[derive(Debug)]
pub enum ASTAtom {
    Int(i32),
    Var(String),
}

#[derive(Debug)]
pub struct OptionallyTypedIdent {
    pub name: String,
    pub ty: Option<Type>,
}

impl OptionallyTypedIdent {
    pub fn new(name: String, ty: Option<Type>) -> Self {
        OptionallyTypedIdent { name, ty }
    }
}