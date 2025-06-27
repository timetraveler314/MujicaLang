use crate::core::{knf, uncurry, Atom, CoreError, TypedAtom};
use crate::frontend::ast::ASTAtom;
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;
use crate::util::name_generator::NameGenerator;

pub struct AST2KNF {
    name_generator: NameGenerator,
}

impl AST2KNF {
    pub fn new() -> Self {
        AST2KNF {
            name_generator: NameGenerator::new("a2k_"),
        }
    }
    
    pub fn convert(
        &mut self,
        expr: uncurry::Expr,
    ) -> knf::Expr {
        match expr {
            uncurry::Expr::Atom { atom, ty } => knf::Expr::Atom(TypedAtom { atom, ty}),
            uncurry::Expr::If { cond, then, else_, ty } => {
                let cond = self.convert(*cond);
                let then = self.convert(*then);
                let else_ = self.convert(*else_);
                
                let cond_var = self.name_generator.fresh_ident();
                
                knf::Expr::Let {
                    bind: cond_var.clone(),
                    value: Box::new(cond),
                    body: Box::new(knf::Expr::If {
                        cond: ASTAtom::Var(cond_var),
                        then: Box::new(then),
                        else_: Box::new(else_),
                        ty: ty.clone(),
                    }),
                    ty,
                    is_polymorphic: false,
                }
            },
            uncurry::Expr::Let { bind, value, body, ty, is_polymorphic } => knf::Expr::Let {
                bind: bind.clone(),
                value: Box::new(self.convert(*value)),
                body: Box::new(self.convert(*body)),
                ty,
                is_polymorphic,
            },
            uncurry::Expr::Apply { func, args, ty } => {
                let func_ty = func.ty();

                // 判断 func 是否是原子表达式
                let (func_atom, func_let_opt) = match *func {
                    uncurry::Expr::Atom { ref atom, ty: ref atom_ty } => {
                        // 是 atom，无需 let
                        (
                            TypedAtom { atom: atom.clone(), ty: atom_ty.clone() },
                            None,
                        )
                    }
                    _ => {
                        // 非 atom，必须 let-bind
                        let func_ident = self.name_generator.fresh_ident();
                        let atom = Atom::Var(func_ident.clone());
                        let func_atom = TypedAtom { atom, ty: func_ty.clone() };
                        let func_let = knf::Expr::Let {
                            bind: func_ident,
                            value: Box::new(self.convert(*func)),
                            body: Box::new(knf::Expr::Atom(TypedAtom {
                                atom: Atom::Unit,
                                ty: Ty::Unit
                            })), // 占位，稍后补上
                            ty: func_ty.clone(),
                            is_polymorphic: false,
                        };
                        (func_atom, Some(func_let))
                    }
                };

                // args 总是 let-bind，因为它们一定要变成变量
                let intermediate_vars: Vec<(ResolvedIdent, Ty)> = args
                    .iter()
                    .map(|arg| (self.name_generator.fresh_ident(), arg.ty()))
                    .collect();

                let result = knf::Expr::Apply {
                    func: func_atom,
                    args: intermediate_vars.iter().map(|(var, ty)| TypedAtom {
                        atom: Atom::Var(var.clone()),
                        ty: ty.clone(),
                    }).collect(),
                    ty: ty.clone(),
                };

                let args_let = args
                    .into_iter()
                    .zip(intermediate_vars)
                    .fold(
                        result,
                        |acc, (arg, (var, ty))| knf::Expr::Let {
                            bind: var,
                            value: Box::new(self.convert(arg)),
                            body: Box::new(acc),
                            ty: ty.clone(),
                            is_polymorphic: false,
                        },
                    );

                // 如果 func 是 atom，则直接返回 args_let
                if let Some(mut func_let) = func_let_opt {
                    // 替换 func_let 的 body 为 args_let
                    if let knf::Expr::Let { ref mut body, .. } = func_let {
                        *body = Box::new(args_let);
                    }
                    func_let
                } else {
                    args_let
                }
            }
            uncurry::Expr::Lambda { args, body, ret_ty } => knf::Expr::Lambda {
                args: args.clone(),
                body: Box::new(self.convert(*body)),
                ret_ty,
            },
        }
    }
}