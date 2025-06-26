use crate::core::CoreError;
use crate::frontend::ast::{ASTAtom, ASTExpr};
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;
use crate::frontend::tyck::tyck::TypedASTExpr;

#[derive(Debug)]
pub enum Expr {
    Atom {
        atom: ASTAtom<ResolvedIdent>,
        ty: Ty
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
        ty: Ty,
    },
    Let {
        bind: ResolvedIdent,
        value: Box<Expr>,
        body: Box<Expr>,
        ty: Ty,
    },
    Apply {
        func: Box<Expr>,
        args: Vec<Expr>,
        ty: Ty,
    },
    Lambda {
        args: Vec<(ResolvedIdent, Ty)>,
        body: Box<Expr>,
        ret_ty: Ty,
    },
}

impl Expr {
    pub fn ty(&self) -> Ty {
        match self {
            Expr::Atom { ty, .. } => ty.clone(),
            Expr::If { ty, .. } => ty.clone(),
            Expr::Let { ty, .. } => ty.clone(),
            Expr::Apply { ty, .. } => ty.clone(),
            Expr::Lambda { ret_ty, .. } => ret_ty.clone(),
        }
    }
}

pub fn uncurry(ast: TypedASTExpr) -> Result<Expr, CoreError> {
    match ast {
        ASTExpr::Atom(atom, ty) => Ok(Expr::Atom { atom, ty }),

        ASTExpr::If { cond, then, else_, ty } => Ok(Expr::If {
            cond: Box::new(uncurry(*cond)?),
            then: Box::new(uncurry(*then)?),
            else_: Box::new(uncurry(*else_)?),
            ty,
        }),

        ASTExpr::Let { bind: (id, _), value, body, ty } => Ok(Expr::Let {
            bind: id,
            value: Box::new(uncurry(*value)?),
            body: Box::new(uncurry(*body)?),
            ty,
        }),

        ASTExpr::Apply { func, args, ty } => {
            let mut func_expr = *func;
            let mut arg_list = vec![*args];

            // Flatten curried apply chain
            while let ASTExpr::Apply { func, args, ty: _ } = func_expr {
                arg_list.push(*args);
                func_expr = *func;
            }

            arg_list.reverse(); // restore application order
            Ok(Expr::Apply {
                func: Box::new(uncurry(func_expr)?),
                args: arg_list.into_iter().map(uncurry).collect::<Result<Vec<_>, _>>()?,
                ty,
            })
        }

        ASTExpr::Lambda { arg, body, ret_ty } => {
            let mut args = vec![arg];
            let mut curr_body = *body;
            let mut final_ret_ty = ret_ty.clone();

            // Flatten nested lambdas
            while let ASTExpr::Lambda { arg, body, ret_ty } = curr_body {
                args.push(arg);
                curr_body = *body;
                final_ret_ty = ret_ty;
            }

            Ok(Expr::Lambda {
                args,
                body: Box::new(uncurry(curr_body)?),
                ret_ty: final_ret_ty,
            })
        }
    }
}
