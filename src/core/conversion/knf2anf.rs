use crate::core::{anf, knf, CoreError};

pub fn knf2anf(knf: knf::Expr) -> Result<anf::Expr, CoreError> {
    // the lifted `id` function
    knf2anf_impl(knf, Box::new(|c| Ok(anf::Expr::CExpr(c))))
}

fn knf2anf_impl(knf: knf::Expr, k: Box<dyn FnOnce(anf::CExpr) -> Result<anf::Expr, CoreError>>) -> Result<anf::Expr, CoreError> {
    match knf {
        knf::Expr::Let { bind, value, body, ty } => {
            let bind_clone = bind.clone();

            Ok(
                knf2anf_impl(
                    *value,
                    Box::from(|c| {
                        Ok(anf::Expr::Let {
                            bind: bind_clone,
                            value: Box::new(c),
                            body: Box::from(knf2anf_impl(*body, k)?),
                            ty
                        })
                    })
                )?
            )
        }
        knf::Expr::Atom{ atom, ty } => {
            k(anf::CExpr::Atom { atom, ty })
        }
        knf::Expr::Apply { func, args, ty } => {
            k(anf::CExpr::Apply { func, args, ty })
        }
        knf::Expr::If { cond, then, else_, ty } => {
            k(anf::CExpr::If { cond, then: Box::from(knf2anf(*then)?), else_: Box::from(knf2anf(*else_)?), ty })
        }
        knf::Expr::Lambda {
            args,
            body,
            ret_ty,
        } => {
            let anf_body = knf2anf_impl(*body, k)?;
            Ok(anf::Expr::CExpr(anf::CExpr::Lambda {
                args,
                body: Box::new(anf_body),
                ret_ty,
            }))
        }
    }
}