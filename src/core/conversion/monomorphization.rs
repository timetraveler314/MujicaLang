use std::collections::{HashMap, HashSet};
use crate::core::{anf, Atom, TypedAtom};
use crate::core::anf::CExpr;
use crate::frontend::name_resolution::{NameIdentifier, ResolvedIdent};
use crate::frontend::ty::{Ty, TypeVar};

pub struct Monomorphization {
    pub instances: HashMap<NameIdentifier, HashMap<Vec<Ty>, (ResolvedIdent, Ty)>>,
    pub polymorphic: HashSet<NameIdentifier>,
}

pub type UpdateMap = HashMap<TypeVar, Ty>;

fn apply_update(ty: Ty, update: &UpdateMap) -> Ty {
    match ty {
        Ty::Mono(var) => {
            if let Some(substituted_ty) = update.get(&var) {
                substituted_ty.clone()
            } else {
                Ty::Mono(var)
            }
        }
        Ty::Arrow(left, right) => {
            Ty::Arrow(
                Box::new(apply_update(*left, update)),
                Box::new(apply_update(*right, update)),
            )
        }
        _ => ty,
    }
}

fn bind(update: &mut UpdateMap, var: &TypeVar, ty: Ty) {
    update.insert(var.clone(), ty);
}

impl Monomorphization {
    pub fn new() -> Self {
        Monomorphization {
            instances: HashMap::new(),
            polymorphic: HashSet::new(),
        }
    }
    
    pub fn make_mono_ident(base: &ResolvedIdent, args: &[Ty]) -> ResolvedIdent {
        let mangled = args.iter().map(|ty| ty.mangle()).
            collect::<Vec<_>>()
            .join("_");
        let name = format!("{}__{}", base.name, mangled);
        let id = format!("{}__{}", base.id.0, mangled);
        
        ResolvedIdent::new(name, id)
    }

    pub fn collect_instances(&mut self, expr: &anf::Expr) {
        match expr {
            anf::Expr::Let { bind, value, body, is_polymorphic, .. } => {
                if *is_polymorphic {
                    self.polymorphic.insert(bind.id.clone());
                }
                
                self.collect_cexpr(&value);
                self.collect_instances(body);
            }
            anf::Expr::CExpr(cexpr) => self.collect_cexpr(&cexpr),
        }
    }

    fn collect_cexpr(&mut self, cexpr: &CExpr) {
        match cexpr {
            CExpr::If { cond: _cond, then, else_, .. } => {
                self.collect_instances(then);
                self.collect_instances(else_);
            }
            CExpr::Apply { func, args: _args, .. } => {
                // match &func.atom {
                //     Atom::Var(var) => {
                //         let arg_types: Vec<Ty> = args.iter().map(|arg| arg.ty.clone()).collect();
                //         
                //         println!("Collecting polymorphic instance for {} with args {:?}", var, arg_types);
                //         
                //         // self.instances
                //         //     .entry(var.id.clone())
                //         //     .or_default()
                //         //     .push((arg_types, make_mono_ident(id, &arg_types)));
                //     }
                //     _ => {} // No polymorphism in other types of atom
                // }
                self.collect_typed_atom(func);
            }
            CExpr::Lambda { args: _args, body, .. } => {
                // Collect instances for the body of the lambda
                self.collect_instances(body);
            }
            CExpr::Atom(typed_atom) => {
                self.collect_typed_atom(typed_atom);
            }
        }
    }

    fn collect_typed_atom(&mut self, typed_atom: &TypedAtom) {
        match &typed_atom.atom {
            Atom::Var(var) => {
                let (arg_types, ret_ty) = typed_atom.ty.extract_args();

                let mono_ident = Self::make_mono_ident(&var, &arg_types);

                println!("Collecting instance for {} with args {:?}, assigning to {:?}",
                         var.id.0, arg_types, mono_ident);

                self.instances
                    .entry(var.id.clone())
                    .or_default()
                    .insert(arg_types, (mono_ident, ret_ty));
            }
            _ => {}
        }
    }

    pub fn rewrite_expr(&self, expr: anf::Expr, update: &mut UpdateMap) -> anf::Expr {
        fn unify(a: Ty, b: Ty, update: &mut UpdateMap) {
            let a = apply_update(a, update);
            let b = apply_update(b, update);

            match (a, b) {
                (Ty::Int, Ty::Int) | (Ty::Bool, Ty::Bool) => {},
                (Ty::Mono(ref x), t) | (t, Ty::Mono(ref x)) => bind(update, x, t),
                (Ty::Arrow(a1, a2), Ty::Arrow(b1, b2)) => {
                    unify(*a1, *b1, update);
                    unify(*a2, *b2, update);
                }
                (a, b) => panic!(
                    "Cannot unify types: {} and {}",
                    a, b
                ),
            }
        }
        
        match expr {
            anf::Expr::Let { bind, value, body, ty, is_polymorphic } => {
                if let CExpr::Lambda { args, body: lam_body, ret_ty } = value.as_ref() {
                    if is_polymorphic {
                        // Polymorphic function, wrap the body with multiple instances
                        let mut new_body = self.rewrite_expr(*body, update);

                        let insts = self.instances.get(&bind.id).unwrap();

                        for (arg_types, (mono_id, actual_ret_ty)) in insts.iter() {
                            let new_args: Vec<(ResolvedIdent, Ty)> = args.iter()
                                .zip(arg_types)
                                .map(|((name, _), t)| (name.clone(), t.clone()))
                                .collect();
                            
                            update.clear();

                            for ((_, inst_ty), (_, arg_ty)) in new_args.iter().zip(args) {
                                // Update the type variable mapping
                                unify(arg_ty.clone(), inst_ty.clone(), update);
                            }

                            println!("Update hashmap is {:?}", update);

                            let new_lambda = CExpr::Lambda {
                                args: new_args,
                                body: Box::new(self.rewrite_expr(*lam_body.clone(), update)),
                                ret_ty: actual_ret_ty.clone(),
                            };

                            new_body = anf::Expr::Let {
                                bind: mono_id.clone(),
                                value: Box::new(new_lambda),
                                body: Box::new(new_body),
                                ty: apply_update(ty.clone(), update),
                                is_polymorphic: false
                            };
                        }

                        return new_body;
                    }
                }
                
                // Not polymorphic, rewrite normally
                anf::Expr::Let {
                    bind: bind.clone(),
                    value: Box::new(self.rewrite_cexpr(*value, update)),
                    body: Box::new(self.rewrite_expr(*body, update)),
                    ty: apply_update(ty, update),
                    is_polymorphic: false,
                }
            }
            anf::Expr::CExpr(cexpr) => anf::Expr::CExpr(self.rewrite_cexpr(cexpr, update)),
        }
    }

    fn rewrite_cexpr(&self, cexpr: CExpr, update: &mut UpdateMap) -> CExpr {
        match cexpr {
            CExpr::Apply { func, args, ty } => {
                let func = self.rewrite_typed_atom(func, update);
                let args = args.into_iter()
                    .map(|arg| self.rewrite_typed_atom(arg, update))
                    .collect();
                
                CExpr::Apply {
                    func,
                    args,
                    ty: apply_update(ty, update),
                }
            }
            CExpr::If { cond, then, else_, ty } => CExpr::If {
                cond,
                then: Box::new(self.rewrite_expr(*then, update)),
                else_: Box::new(self.rewrite_expr(*else_, update)),
                ty: apply_update(ty, update),
            },
            CExpr::Atom(typed_atom) => {
                let rewritten_atom = self.rewrite_typed_atom(typed_atom, update);
                CExpr::Atom(rewritten_atom)
            }
            CExpr::Lambda { args, body, ret_ty } => {
                let rewritten_body = self.rewrite_expr(*body, update);
                
                CExpr::Lambda {
                    args,
                    body: Box::new(rewritten_body),
                    ret_ty: apply_update(ret_ty, update), // TODO
                }
            }
        }
    }
    
    fn rewrite_typed_atom(&self, typed_atom: TypedAtom, update: &mut UpdateMap) -> TypedAtom {
        match &typed_atom.atom {
            Atom::Var(var) => {
                if let Some(insts) = self.instances.get(&var.id) {
                    if self.polymorphic.contains(&var.id) {
                        let (new_ident, _) = insts.get(&typed_atom.ty.extract_args().0).unwrap();
                        
                        return TypedAtom {
                            atom: Atom::Var(new_ident.clone()),
                            ty: apply_update(typed_atom.ty, update),
                        };
                    }
                }
            }
            _ => {}
        }
        
        TypedAtom {
            atom: typed_atom.atom,
            ty: apply_update(typed_atom.ty, update),
        }
    }
}