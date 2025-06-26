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
                let func_ident = self.name_generator.fresh_ident();

                let intermediate_vars: Vec<(ResolvedIdent, Ty)> = args
                    .iter()
                    .map(|arg| (self.name_generator.fresh_ident(), arg.ty()))
                    .collect();

                let result = knf::Expr::Apply {
                    func: TypedAtom {
                        atom: Atom::Var(func_ident.clone()),
                        ty: func_ty.clone(),
                    },
                    args: intermediate_vars.iter().map(|(var, ty)| TypedAtom { atom: Atom::Var(var.clone()), ty: ty.clone() }).collect(),
                    ty: ty.clone(),
                };

                let let_surrounded = args
                    .into_iter()
                    .zip(intermediate_vars)
                    .fold(
                        result,
                        |acc, (arg, (var, ty))| knf::Expr::Let {
                            bind: var,
                            value: Box::new(self.convert(arg)),
                            body: Box::new(acc),
                            ty: ty.clone(),
                            is_polymorphic: false
                        },
                    );

                knf::Expr::Let {
                    bind: func_ident,
                    value: Box::new(self.convert(*func)),
                    body: Box::new(let_surrounded),
                    ty,
                    is_polymorphic: false,
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