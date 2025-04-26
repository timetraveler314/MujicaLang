use crate::core::{ast, knf};
use crate::core::ast::Expr;
use crate::core::common::Atom;
use crate::core::ty::TypedIdent;

pub fn ast2knf(expr: ast::Expr) -> knf::Expr {
    let mut env = Ast2KnfEnv::new();
    ast2knf_impl(expr, &mut env)
}

struct Ast2KnfEnv {
    /// counter for generating unique variable names
    counter: usize,
}

impl Ast2KnfEnv {
    fn new() -> Self {
        Ast2KnfEnv { counter: 0 }
    }

    fn next_var(&mut self) -> String {
        let var = format!("v{}", self.counter);
        self.counter += 1;
        var
    }
}

fn ast2knf_impl(
    expr: ast::Expr,
    env: &mut Ast2KnfEnv,
) -> knf::Expr {
    match expr {
        ast::Expr::Atom(atom) => knf::Expr::Atom(atom),
        ast::Expr::Let { bind, value, body } => knf::Expr::Let {
            bind: bind.clone(),
            value: Box::new(ast2knf_impl(*value, env)),
            body: Box::new(ast2knf_impl(*body,env)),
        },
        ast::Expr::Op { op, args } => {
            let intermediate_vars: Vec<_> = args
                .iter()
                .map(|arg| TypedIdent { name: env.next_var(), ty: arg.ty() })
                .collect();
            
            let result = knf::Expr::Op {
                op,
                args: intermediate_vars.iter().map(|v| Atom::Var(v.clone())).collect(),
            };
            
            let let_surrounded = args
                .into_iter()
                .zip(intermediate_vars)
                .fold(
                    result,
                    |acc, (arg, var)| knf::Expr::Let {
                        bind: var.clone(),
                        value: Box::new(ast2knf_impl(arg, env)),
                        body: Box::new(acc),
                    },
                );
            
            let_surrounded
        }
        Expr::If { cond, then, else_, ty } => {
            let cond_ty = cond.ty();
            
            let cond = ast2knf_impl(*cond, env);
            let then = ast2knf_impl(*then, env);
            let else_ = ast2knf_impl(*else_, env);
            
            let cond_var = TypedIdent { name: env.next_var(), ty: cond_ty };
            
            knf::Expr::Let {
                bind: cond_var.clone(),
                value: Box::new(cond),
                body: Box::new(knf::Expr::If {
                    cond: Box::new(Atom::Var(cond_var)),
                    then: Box::new(then),
                    else_: Box::new(else_),
                    ty,
                }),
            }
        }
        Expr::LetFun { bind, args, body, body2 } => knf::Expr::LetFun {
            bind: bind.clone(),
            args: args.clone(),
            body: Box::new(ast2knf_impl(*body, env)),
            body2: Box::new(ast2knf_impl(*body2, env)),
        },
        Expr::Call { closure, args, ret_ty } => {
            // similar to the `Op` case, we need to create intermediate variables for the arguments
            let closure_ty = closure.ty();
            let closure_var = TypedIdent { name: env.next_var(), ty: closure_ty };
            
            let intermediate_vars: Vec<_> = args
                .iter()
                .map(|arg| TypedIdent { name: env.next_var(), ty: arg.ty() })
                .collect();
            
            let result = knf::Expr::Call {
                closure: Atom::Var(closure_var.clone()),
                args: intermediate_vars.iter().map(|v| Atom::Var(v.clone())).collect(),
                ret_ty,
            };
            
            let let_surrounded = args
                .into_iter()
                .zip(intermediate_vars)
                .fold(
                    result,
                    |acc, (arg, var)| knf::Expr::Let {
                        bind: var.clone(),
                        value: Box::new(ast2knf_impl(arg, env)),
                        body: Box::new(acc),
                    },
                );
            
            knf::Expr::Let {
                bind: closure_var.clone(),
                value: Box::new(ast2knf_impl(*closure, env)),
                body: Box::new(let_surrounded),
            }
        }
    }
}