use std::rc::Rc;
use crate::core::{Atom, TypedAtom};
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Closure {
    pub global_name: String,
    pub ret_ty: Ty,
    pub capture: Vec<(ResolvedIdent, Ty)>,
    pub args: Vec<(ResolvedIdent, Ty)>,
}

#[derive(Debug, Clone)]
pub enum ClosureCExpr {
    Atom(TypedAtom),
    If {
        cond: Atom,
        then: Box<ClosureExpr>,
        else_: Box<ClosureExpr>,
        ty: Ty,
    },
    Apply {
        func: TypedAtom,
        args: Vec<TypedAtom>,
        ty: Ty,
    },
    Closure(Rc<Closure>),
}

#[derive(Debug, Clone)]
pub enum ClosureExpr {
    CExpr(ClosureCExpr),
    Let {
        bind: ResolvedIdent,
        value: Box<ClosureCExpr>,
        body: Box<ClosureExpr>,
        ty: Ty,
    },
}
