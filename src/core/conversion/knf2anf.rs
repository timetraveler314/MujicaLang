use crate::backend::Error;
use crate::core::{anf, knf};

pub fn knf2anf(knf: knf::Expr) -> Result<anf::Expr, Error> {
    // the lifted `id` function
    knf2anf_impl(knf, Box::new(|c| Ok(anf::Expr::CExpr(c))))
}

fn knf2anf_impl(knf: knf::Expr, k: Box<dyn FnOnce(anf::CExpr) -> Result<anf::Expr, Error>>) -> Result<anf::Expr, Error> {
    match knf {
        knf::Expr::Let { bind, value, body } => {
            let bind_clone = bind.clone();
            
            Ok(
                knf2anf_impl(
                    *value,
                    Box::from(|c| {
                        Ok(anf::Expr::Let {
                            bind: bind_clone,
                            value: Box::new(c),
                            body: Box::from(knf2anf_impl(*body, k)?)
                        })
                    })
                )?
            )
        }
        knf::Expr::Atom(atom) => {
            k(anf::CExpr::Atom(atom))
        }
        knf::Expr::Op { op, args } => {
            k(anf::CExpr::Op { op, args })
        }
        knf::Expr::Call { closure, args, ret_ty } => {
            k(anf::CExpr::Call { closure, args, ret_ty })
        }
        knf::Expr::If { cond, then, else_, ty } => {
            k(anf::CExpr::If { cond, then: Box::from(knf2anf(*then)?), else_: Box::from(knf2anf(*else_)?), ty })
        }
        knf::Expr::LetFun { bind, args, body, body2 } => {
            k(anf::CExpr::LetFun {
                bind,
                args,
                body: Box::from(knf2anf(*body)?),
                body2: Box::from(knf2anf(*body2)?)
            })
        }
    }
}