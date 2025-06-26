use crate::core::common::{Atom, OpType};
use crate::core::ty::{Type, TypedIdent};

#[derive(Debug)]
pub enum Expr {
    Atom(Atom),
    Op {
        op: OpType,
        args: Vec<Atom>,
    },
    If {
        cond: Box<Atom>,
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
        closure: Atom,
        args: Vec<Atom>,
        ret_ty: Type,
    },
}