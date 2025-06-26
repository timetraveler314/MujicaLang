use crate::core::Atom;
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;

#[derive(Debug)]
pub enum CExpr {
    Atom {
        atom: Atom,
        ty: Ty
    },
    If {
        cond: Atom,
        then: Box<Expr>,
        else_: Box<Expr>,
        ty: Ty,
    },
    Apply {
        func: Atom,
        args: Vec<Atom>,
        ty: Ty,
    },
    Lambda {
        args: Vec<(ResolvedIdent, Ty)>,
        body: Box<Expr>,
        ret_ty: Ty,
    },
}

#[derive(Debug)]
pub enum Expr {
    CExpr(CExpr),
    Let {
        bind: ResolvedIdent,
        value: Box<CExpr>,
        body: Box<Expr>,
        ty: Ty
    },
}