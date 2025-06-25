use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use crate::core::ty::Type;
use crate::frontend::ast::{ASTAtom, InputASTExpr, OpType};
use crate::frontend::FrontendError;
use crate::frontend::name_resolution::{ResolvedASTExpr, ResolvedIdent};
use crate::frontend::ty::{Scheme, Ty, TypeVar, TypingContext};
use crate::frontend::tyck::subst::apply_subst;
use crate::frontend::tyck::type_class::TypeClassConstraint;

#[derive(Debug)]
pub struct TypeChecker {
    context: TypingContext,
    fresh: usize,
    subst: HashMap<TypeVar, Ty>,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            context: TypingContext::new(),
            fresh: 0,
            subst: HashMap::new(),
        }
    }

    fn fresh_ty(&mut self) -> Ty {
        let v = format!("t{}", self.fresh);
        self.fresh += 1;
        Ty::Mono(v)
    }

    fn apply_subst(&self, ty: Ty) -> Ty {
        match ty {
            Ty::Mono(ref v) => {
                if let Some(t) = self.subst.get(v) {
                    self.apply_subst(t.clone())
                } else {
                    ty
                }
            }
            Ty::Arrow(a, b) => {
                Ty::Arrow(
                    Box::new(self.apply_subst(*a)),
                    Box::new(self.apply_subst(*b)),
                )
            }
            _ => ty,
        }
    }

    fn unify(&mut self, a: Ty, b: Ty) -> Result<(), FrontendError> {
        let a = self.apply_subst(a);
        let b = self.apply_subst(b);

        match (a, b) {
            (Ty::Int, Ty::Int) | (Ty::Bool, Ty::Bool) => Ok(()),
            (Ty::Mono(ref x), t) | (t, Ty::Mono(ref x)) => self.bind(x, t),
            (Ty::Arrow(a1, a2), Ty::Arrow(b1, b2)) => {
                self.unify(*a1, *b1)?;
                self.unify(*a2, *b2)
            }
            (a, b) => Err(FrontendError::TypeError(format!(
                "Cannot unify types: {} and {}",
                a, b
            ))),
        }
    }

    fn bind(&mut self, var: &TypeVar, ty: Ty) -> Result<(), FrontendError> {
        if let Ty::Mono(ref x) = ty {
            if var == x {
                return Ok(()); // trivial
            }
        }

        if self.occurs_check(var, &ty) {
            return Err(FrontendError::TypeError(format!(
                "Cannot unify {} with {}, occurs check failed",
                var, ty
            )));
        }

        self.subst.insert(var.to_string(), ty);
        Ok(())
    }

    fn occurs_check(&self, var: &str, ty: &Ty) -> bool {
        match ty {
            Ty::Mono(x) => {
                if x == var {
                    true
                } else if let Some(t) = self.subst.get(x) {
                    self.occurs_check(var, t)
                } else {
                    false
                }
            }
            Ty::Arrow(a, b) => self.occurs_check(var, a) || self.occurs_check(var, b),
            _ => false,
        }
    }


    /// Instantiate a type scheme by replacing its type variables with fresh type variables.
    pub fn instantiate(&mut self, scheme: &Scheme) -> Ty {
        let mut subst = HashMap::new();
        for var in &scheme.vars {
            let fresh_var = self.fresh_ty();
            subst.insert(var.clone(), fresh_var);
        }

        apply_subst(scheme.ty.clone(), subst)
    }

    pub fn infer(&mut self, expr: &mut ResolvedASTExpr) -> Result<Ty, FrontendError> {
        let primitive = match expr {
            ResolvedASTExpr::Atom(atom, atom_ty) => {
                match atom {
                    ASTAtom::Int(_) => Ok(Ty::Int),
                    ASTAtom::Var(ident) => {
                        // lookup in context
                        let scheme = self.context.get(&ident.id).ok_or_else(|| {
                            FrontendError::UnboundVariable(ident.name.clone())
                        })?.clone();

                        // instantiate the type scheme
                        let ty = self.instantiate(&scheme);
                        
                        // Set the type of the atom
                        *atom_ty = Some(ty.clone());

                        Ok(ty)
                    }
                    ASTAtom::Op(op) => {
                        match op {
                            OpType::Add | OpType::Sub | OpType::Mul | OpType::Div => {
                                // Arithmetic operations expect two integers for now
                                Ok(Ty::Arrow(
                                    Box::new(Ty::Int),
                                    Box::new(Ty::Arrow(Box::new(Ty::Int), Box::new(Ty::Int))),
                                ))
                            },
                            OpType::Eq | OpType::Neq | OpType::Gt | OpType::Lt | OpType::Geq | OpType::Leq => {
                                // Equality operations now only work with integers
                                Ok(Ty::Arrow(
                                    Box::new(Ty::Int),
                                    Box::new(Ty::Arrow(Box::new(Ty::Int), Box::new(Ty::Bool))),
                                ))
                            }
                        }
                    },
                }
            }
            ResolvedASTExpr::If { cond, then, else_, ty } => {
                let cond_ty = self.infer(cond)?;

                self.unify(cond_ty, Ty::Bool)?;

                let then_ty = self.infer(then)?;
                let else_ty = self.infer(else_)?;

                self.unify(then_ty.clone(), else_ty)?;
                
                let then_ty = self.apply_subst(then_ty);

                *ty = Some(then_ty.clone());
                Ok(then_ty)
            }
            ResolvedASTExpr::Let { bind: (ident, scheme), value, body, ty } => {
                match scheme {
                    None => {
                        // Infer the type ourself
                        let value_ty = self.infer(value)?;

                        // Inferred. Insert into context
                        self.context.insert(
                            ident.id,
                            Scheme {
                                ty: value_ty.clone(),
                                constraints: vec![],
                                vars: vec![],
                            },
                        )
                    }
                    Some(scheme) => {
                        let instantiated_ty = self.instantiate(scheme);
                        
                        // Check the value against the instantiated type
                        self.check(value, &instantiated_ty)?;
                        
                        // Passed. Insert into context
                        self.context.insert(
                            ident.id,
                            scheme.clone()
                        );
                    }
                }
                
                // Infer the type of the body
                let body_ty = self.infer(body)?;
                
                let body_ty = self.apply_subst(body_ty);
                
                // Set the type of the let expression
                *ty = Some(body_ty.clone());
                Ok(body_ty)
            }
            ResolvedASTExpr::Apply { func, args, ty } => {
                let func_ty = self.infer(func)?;
                let arg_ty = self.infer(args)?;
                
                let ret_ty = self.fresh_ty();
                
                // Unify the function type with the expected type
                self.unify(func_ty, Ty::Arrow(Box::new(arg_ty), Box::new(ret_ty.clone())))?;
                
                let ret_ty = self.apply_subst(ret_ty);
                
                // Set the type of the application
                *ty = Some(ret_ty.clone());
                
                // Return the return type of the function
                Ok(ret_ty)
            }
            ResolvedASTExpr::Lambda { arg: (ident, ty_opt), body, ret_ty } => {
                // In infer mode, we expect the argument type to be provided
                let arg_ty = ty_opt.clone().ok_or_else(|| {
                    FrontendError::TypeError("Lambda argument type is not provided".to_string())
                })?;
                
                // Bind the argument type in the context
                self.context.insert(
                    ident.id,
                    Scheme {
                        ty: arg_ty.clone(),
                        constraints: vec![],
                        vars: vec![],
                    },
                );
                
                // Infer the body type
                let body_ty = self.infer(body)?;
                
                let body_ty = self.apply_subst(body_ty);
                
                // set the return type of the lambda
                *ret_ty = Some(body_ty.clone());
                
                Ok(Ty::Arrow(
                    Box::new(arg_ty),
                    Box::new(body_ty.clone()),
                ))
            }
        }?;

        let substituted = self.apply_subst(primitive);

        Ok(substituted)
    }
    
    pub fn check(&mut self, expr: &mut ResolvedASTExpr, expected: &Ty) -> Result<(), FrontendError> {
        match expr {
            ResolvedASTExpr::Lambda { arg: (ident, ty_opt), body, ret_ty } => {
                if let Ty::Arrow(arg_expected, ret_expected) = expected {
                    // Insert the argument type into the context
                    self.context.insert(
                        ident.id,
                        Scheme {
                            ty: *arg_expected.clone(),
                            constraints: vec![],
                            vars: vec![],
                        },
                    );
                    
                    // Check the body against the return type
                    self.check(body, ret_expected)?;
                    
                    // Success, set the type of the lambda
                    let ret_expected = self.apply_subst(*ret_expected.clone());
                    let arg_expected = self.apply_subst(*arg_expected.clone());
                    
                    *ret_ty = Some(ret_expected);
                    *ty_opt = Some(arg_expected);
                    
                    Ok(())
                } else {
                    Err(FrontendError::TypeError(format!(
                        "Expected a function type, found: {}",
                        expected
                    )))
                }
            },
            _ => {
                // For other expressions, we just infer and unify
                let inferred = self.infer(expr)?;
                self.unify(inferred, expected.clone())?;

                // After successful unification, apply substitution to expected type
                let final_ty = self.apply_subst(expected.clone());

                // Store the type in the AST node
                match expr {
                    ResolvedASTExpr::Atom(atom, ty) => {
                        *ty = Some(final_ty);
                    }
                    ResolvedASTExpr::If { ty, .. } => {
                        *ty = Some(final_ty);
                    }
                    ResolvedASTExpr::Let { ty, .. } => {
                        *ty = Some(final_ty);
                    }
                    ResolvedASTExpr::Apply { ty, .. } => {
                        *ty = Some(final_ty);
                    }
                    ResolvedASTExpr::Lambda { ret_ty, .. } => {
                        *ret_ty = Some(final_ty);
                    }
                }

                Ok(())
            }
        }
    }
    
    pub fn final_apply(&self, ast: &mut ResolvedASTExpr) {
        match ast {
            ResolvedASTExpr::Atom(_, ty) => {
                if let Some(t) = ty {
                    *ty = Some(self.apply_subst(t.clone()));
                }
            }
            ResolvedASTExpr::If { cond, then, else_, ty } => {
                *ty = Some(self.apply_subst(ty.clone().unwrap()));
                self.final_apply(cond);
                self.final_apply(then);
                self.final_apply(else_);
            }
            ResolvedASTExpr::Let { value, body, ty, .. } => {
                *ty = Some(self.apply_subst(ty.clone().unwrap()));
                self.final_apply(value);
                self.final_apply(body);
            }
            ResolvedASTExpr::Apply { func, args, ty } => {
                *ty = Some(self.apply_subst(ty.clone().unwrap()));
                self.final_apply(func);
                self.final_apply(args);
            }
            ResolvedASTExpr::Lambda { body, ret_ty, .. } => {
                *ret_ty = Some(self.apply_subst(ret_ty.clone().unwrap()));
                self.final_apply(body);
            }
        }
    }
}

impl Display for TypeChecker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display typing context
        writeln!(f, "TypeChecker State:")?;
        writeln!(f, "=================")?;
        writeln!(f, "Typing Context:")?;

        if self.context.is_empty() {
            writeln!(f, "  <empty>")?;
        } else {
            for (id, scheme) in self.context.get_mapping() {
                writeln!(f, "  {} => {}", id, scheme)?;
            }
        }

        // Display substitution map
        writeln!(f, "\nSubstitutions:")?;
        if self.subst.is_empty() {
            writeln!(f, "  <empty>")?;
        } else {
            for (var, ty) in &self.subst {
                writeln!(f, "  {} ↦ {}", var, ty)?;
            }
        }

        // Display fresh variable counter
        writeln!(f, "\nFresh variable counter: {}", self.fresh)?;

        Ok(())
    }
}