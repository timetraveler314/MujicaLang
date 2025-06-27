use std::collections::{HashMap, HashSet};
use crate::core::{anf, Atom, TypedAtom};
use crate::core::anf::CExpr;
use crate::frontend::name_resolution;
use crate::frontend::name_resolution::{NameIdentifier, ResolvedIdent};
use crate::frontend::ty::Ty;

pub struct Monomorphization {
    pub instances: HashMap<NameIdentifier, HashMap<Vec<Ty>, ResolvedIdent>>,
    pub polymorphic: HashSet<NameIdentifier>,
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
            CExpr::If { cond, then, else_, .. } => {
                self.collect_instances(then);
                self.collect_instances(else_);
            }
            CExpr::Apply { func, args, ty } => {
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
            CExpr::Lambda { args, body, .. } => {
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
                let arg_types: Vec<Ty> = typed_atom.ty.extract_args();
                
                let mono_ident = Self::make_mono_ident(&var, &arg_types);
                
                println!("Collecting instance for {} with args {:?}, assigning to {:?}", 
                         var.id.0, arg_types, mono_ident);

                self.instances
                    .entry(var.id.clone())
                    .or_default()
                    .insert(arg_types, mono_ident);
            }
            _ => {}
        }
    }

    pub fn rewrite_expr(&self, expr: anf::Expr) -> anf::Expr {
        match expr {
            anf::Expr::Let { bind, value, body, ty, is_polymorphic } => {
                if let CExpr::Lambda { args, body: lam_body, ret_ty } = value.as_ref() {
                    if is_polymorphic {
                        // Polymorphic function, wrap the body with multiple instances
                        let insts = self.instances.get(&bind.id).unwrap();
                        
                        let mut new_body = self.rewrite_expr(*body);

                        for (arg_types, mono_id) in insts.iter() {
                            let new_args = args.iter()
                                .zip(arg_types)
                                .map(|((name, _), t)| (name.clone(), t.clone()))
                                .collect();

                            let new_lambda = CExpr::Lambda {
                                args: new_args,
                                body: Box::new(self.rewrite_expr(*lam_body.clone())),
                                ret_ty: ret_ty.clone(), // 可做subst
                            };

                            new_body = anf::Expr::Let {
                                bind: mono_id.clone(),
                                value: Box::new(new_lambda),
                                body: Box::new(new_body),
                                ty: ty.clone(),
                                is_polymorphic: false
                            };
                        }

                        return new_body;
                    }
                }
                
                // Not polymorphic, rewrite normally
                anf::Expr::Let {
                    bind: bind.clone(),
                    value: Box::new(self.rewrite_cexpr(*value)),
                    body: Box::new(self.rewrite_expr(*body)),
                    ty: ty.clone(),
                    is_polymorphic: false,
                }
            }
            anf::Expr::CExpr(cexpr) => anf::Expr::CExpr(self.rewrite_cexpr(cexpr)),
        }
    }

    fn rewrite_cexpr(&self, cexpr: CExpr) -> CExpr {
        match cexpr {
            CExpr::Apply { func, args, ty } => {
                let func = self.rewrite_typed_atom(func);
                let args = args.into_iter()
                    .map(|arg| self.rewrite_typed_atom(arg))
                    .collect();
                
                CExpr::Apply {
                    func,
                    args,
                    ty,
                }
            }
            CExpr::If { cond, then, else_, ty } => CExpr::If {
                cond,
                then: Box::new(self.rewrite_expr(*then)),
                else_: Box::new(self.rewrite_expr(*else_)),
                ty,
            },
            CExpr::Atom(typed_atom) => {
                let rewritten_atom = self.rewrite_typed_atom(typed_atom);
                CExpr::Atom(rewritten_atom)
            }
            CExpr::Lambda { args, body, ret_ty } => {
                let rewritten_body = self.rewrite_expr(*body);
                
                CExpr::Lambda {
                    args,
                    body: Box::new(rewritten_body),
                    ret_ty,
                }
            }
        }
    }
    
    fn rewrite_typed_atom(&self, typed_atom: TypedAtom) -> TypedAtom {
        match &typed_atom.atom {
            Atom::Var(var) => {
                if let Some(insts) = self.instances.get(&var.id) {
                    if self.polymorphic.contains(&var.id) {
                        let new_ident = insts.get(&typed_atom.ty.extract_args()).unwrap();
                        
                        return TypedAtom {
                            atom: Atom::Var(new_ident.clone()),
                            ty: typed_atom.ty,
                        };
                    }
                }
            }
            _ => {}
        }
        typed_atom.clone()
    }
}