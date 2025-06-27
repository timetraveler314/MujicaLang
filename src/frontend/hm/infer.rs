use std::collections::VecDeque;
use crate::frontend::ast::{ASTExpr, InputASTExpr};
use crate::frontend::FrontendError;
use crate::frontend::hm::constraint::Constraint;
use crate::frontend::hm::subst::Subst;
use crate::frontend::hm::TypeError;
pub(crate) use crate::frontend::ty::{MonoContext, Scheme, Ty};

pub struct FreshVarGenerator {
    next_id: usize,
}

impl FreshVarGenerator {
    pub fn new() -> Self {
        FreshVarGenerator { next_id: 0 }
    }

    pub fn fresh_var(&mut self) -> Ty {
        let var_name = format!("t{}", self.next_id);
        self.next_id += 1;
        Ty::Mono(var_name)
    }

    pub fn annotate(&mut self, ty: Option<Ty>) -> Ty {
        ty.unwrap_or_else(|| self.fresh_var())
    }
}

pub fn annotate_type_var(expr: InputASTExpr, fresh_var_generator: &mut FreshVarGenerator) -> Result<ASTExpr<Ty>, FrontendError> {
    match expr {
        InputASTExpr::Atom(atom) => {
            Ok(ASTExpr::Atom(atom))
        }
        InputASTExpr::If { cond, then , else_, ty } => {
            Ok(ASTExpr::If {
                cond: Box::new(annotate_type_var(*cond, fresh_var_generator)?),
                then: Box::new(annotate_type_var(*then, fresh_var_generator)?),
                else_: Box::new(annotate_type_var(*else_, fresh_var_generator)?),
                ty: fresh_var_generator.annotate(ty),
            })
        }
        InputASTExpr::Let { bind: (bind_name, bind_ty), value, body, ty } => {
            Ok(ASTExpr::Let {
                bind: (bind_name, fresh_var_generator.annotate(bind_ty)),
                value: Box::new(annotate_type_var(*value, fresh_var_generator)?),
                body: Box::new(annotate_type_var(*body, fresh_var_generator)?),
                ty: fresh_var_generator.annotate(ty),
            })
        }
        InputASTExpr::Apply { func, args, ty } => {
            Ok(ASTExpr::Apply {
                func: Box::new(annotate_type_var(*func, fresh_var_generator)?),
                args: Box::new(annotate_type_var(*args, fresh_var_generator)?),
                ty: fresh_var_generator.annotate(ty),
            })
        }
        InputASTExpr::Lambda { arg: (arg_name, arg_ty), body, ret_ty } => {
            Ok(ASTExpr::Lambda {
                arg: (arg_name, fresh_var_generator.annotate(arg_ty)),
                body: Box::new(annotate_type_var(*body, fresh_var_generator)?),
                ret_ty: fresh_var_generator.annotate(ret_ty),
            })
        }
    }
}

pub fn extract_constraints(expr: &ASTExpr<Ty>, constraints: &mut Vec<Constraint>, gamma: &mut MonoContext) -> Result<Ty, FrontendError> {
    match expr {
        ASTExpr::Atom(atom) => {
            atom.ty(gamma)
        }
        ASTExpr::If { cond, then, else_, ty } => {
            let cond_ty = extract_constraints(cond, constraints, gamma)?;
            let then_ty = extract_constraints(then, constraints, gamma)?;
            let else_ty = extract_constraints(else_, constraints, gamma)?;

            // Ensure that the condition is a boolean
            constraints.push(Constraint::Eq(cond_ty.clone(), Ty::Bool));

            // `then` and `else` branches must have the same type as the overall type
            constraints.push(Constraint::Eq(then_ty.clone(), ty.clone()));
            constraints.push(Constraint::Eq(else_ty.clone(), ty.clone()));

            Ok(ty.clone())
        }
        ASTExpr::Let { bind, value, body, ty } => {
            // Evaluate the value expression to get its type
            let value_ty = extract_constraints(value, constraints, gamma)?;

            // Bind the variable in the context
            gamma.push(&bind.0, bind.1.clone());

            let body_ty = extract_constraints(body, constraints, gamma)?;

            gamma.pop();

            // Ensure the bound variable has the correct type
            constraints.push(Constraint::Eq(value_ty.clone(), bind.1.clone()));

            // Ensure the body type matches the overall type
            constraints.push(Constraint::Eq(body_ty.clone(), ty.clone()));

            Ok(ty.clone())
        }
        ASTExpr::Apply { func, args, ty } => {
            let func_ty = extract_constraints(func, constraints, gamma)?;
            let args_ty = extract_constraints(args, constraints, gamma)?;

            // func_ty = arg_ty -> ty
            constraints.push(Constraint::Eq(
                func_ty.clone(),
                Ty::Arrow(Box::new(args_ty.clone()), Box::new(ty.clone())),
            ));

            Ok(ty.clone())
        }
        ASTExpr::Lambda { arg: (arg_name, arg_ty), body, ret_ty } => {
            // Bind the argument type in the context
            gamma.push(arg_name, arg_ty.clone());

            let body_ty = extract_constraints(body, constraints, gamma)?;

            gamma.pop();

            // Ensure the body type matches the return type
            constraints.push(Constraint::Eq(body_ty.clone(), ret_ty.clone()));

            // Return the function type
            Ok(Ty::Arrow(Box::new(arg_ty.clone()), Box::new(ret_ty.clone())))
        }
    }
}

pub fn unify(constraints: &[Constraint]) -> Result<Subst, FrontendError> {
    pub fn unify_impl(constraints: &mut VecDeque<Constraint>, subst: &mut Subst) -> Result<(), TypeError> {
        println!("Unifying constraints: {:?}", constraints);
        println!("Current substitution: {:?}", subst);
        
        let constraint = match constraints.pop_front() {
            Some(c) => c,
            None => return Ok(()), // No more constraints to process
        };
        
        match constraint {
            Constraint::Eq(t1, t2) => {
                let t1 = subst.apply(t1.clone());
                let t2 = subst.apply(t2.clone());

                match (t1, t2) {
                    (Ty::Mono(var1), Ty::Mono(var2)) if var1 == var2 => {
                        // Solve the rest recursively
                        unify_impl(constraints, subst)
                    },
                    (Ty::Mono(var), ty) | (ty, Ty::Mono(var)) => {
                        // Occurrence check
                        if ty.contains_var(&var) {
                            return Err(TypeError::UnificationError(format!(
                                "Cannot unify type variable {} with type {} due to occurrence check",
                                var, ty
                            )));
                        }
                        
                        // Apply the new-found substitution on the remaining constraints
                        for constraint in constraints.iter_mut() {
                            *constraint = constraint.apply(&var, &ty);
                        }

                        // Solve the rest recursively
                        unify_impl(constraints, subst)?;
                        
                        // Extend the substitution
                        subst.extend(var, ty)
                    },
                    (Ty::Arrow(left1, right1), Ty::Arrow(left2, right2)) => {
                        // Unify the left and right parts of the arrows
                        constraints.push_front(Constraint::Eq(*left1, *left2));
                        constraints.push_front(Constraint::Eq(*right1, *right2));
                        unify_impl(constraints, subst)
                    }
                    // Base types
                    (Ty::Int, Ty::Int) | (Ty::Bool, Ty::Bool) | (Ty::Unit, Ty::Unit) => Ok(()),
                    // If types are not unifiable
                    (t1, t2) => Err(TypeError::UnificationError(format!(
                        "Cannot unify types: {} and {}",
                        t1, t2
                    ))),
                }
            }
        }
    }

    let mut subst = Subst::empty();
    let mut constraints_queue: VecDeque<Constraint> = constraints.iter().cloned().collect();
    unify_impl(&mut constraints_queue, &mut subst).or_else(|err| {
        Err(FrontendError::TypeError(err))
    })?;

    Ok(subst)
}