use std::collections::HashMap;
use crate::frontend::FrontendError;
use crate::frontend::name_resolution::ResolvedASTExpr;
use crate::frontend::ty::{Scheme, Ty, TypingContext};
use crate::frontend::tyck::subst::apply_subst;
use crate::frontend::tyck::type_class::TypeClassConstraint;

pub struct TypeChecker {
    context: TypingContext,
    fresh: usize,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            context: TypingContext::new(),
            fresh: 0,
        }
    }
    
    fn fresh_ty(&mut self) -> Ty {
        let v = format!("t{}", self.fresh);
        self.fresh += 1;
        Ty::Mono(v)
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
    
    // pub fn infer(&mut self, expr: &mut ResolvedASTExpr) -> Result<(Ty, Vec<TypeClassConstraint>), FrontendError> {
    //     match expr {
    //         ResolvedASTExpr::Atom(_) => {}
    //         ResolvedASTExpr::If { .. } => {}
    //         ResolvedASTExpr::Let { .. } => {}
    //         ResolvedASTExpr::Apply { .. } => {}
    //         ResolvedASTExpr::Lambda { .. } => {}
    //     }
    // }
}