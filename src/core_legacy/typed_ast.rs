pub(crate) use crate::core::common::{Atom, OpType};
use crate::core::ty::{Type, TypedIdent};

#[derive(Debug)]
pub enum Expr {
    Atom(Atom),
    Op {
        op: OpType,
        args: Vec<Expr>,
        ty: Type
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
        ty: Type,
    },
    Let {
        bind: TypedIdent,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    LetFun {
        bind: TypedIdent,
        args: Vec<TypedIdent>,
        body: Box<Expr>,
        body2: Box<Expr>,
    },
    Call {
        closure: Box<Expr>,
        args: Vec<Expr>,
        ret_ty: Type,
    },
}

impl Expr {
    pub fn ty(&self) -> Type {
        match self {
            Expr::Atom(atom) => atom.ty(),
            Expr::Op { ty, .. } => ty.clone(),
            Expr::If { ty, .. } => ty.clone(),
            Expr::Let { value, .. } => value.ty(),
            Expr::LetFun { body2, .. } => body2.ty(),
            Expr::Call { ret_ty, .. } => ret_ty.clone(),
        }
    }
}