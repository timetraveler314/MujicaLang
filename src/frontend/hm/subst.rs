use std::collections::HashMap;
use crate::frontend::hm::TypeError;
use crate::frontend::ty::{Ty, TypeVar};

#[derive(Debug, Clone)]
pub struct Subst {
    mappings: HashMap<TypeVar, Ty>,  // TyVar → Ty
}

impl Subst {
    pub fn empty() -> Self {
        Subst {
            mappings: HashMap::new(),
        }
    }
    
    pub fn extend(&mut self, var: TypeVar, ty: Ty) -> Result<(), TypeError> {
        // Occurrence check: if `ty` contains `var`, we cannot substitute it
        if ty.contains_var(&var) {
            return Err(TypeError::UnificationError(format!(
                "Cannot substitute {} with {}: occurrence check failed",
                var, ty
            )));
        }
        
        // Transitively apply existing substitutions
        let ty = self.apply(ty);
        
        // Insert the new mapping
        self.mappings.insert(var, ty);
        
        Ok(())
    }
    
    pub fn apply(&self, ty: Ty) -> Ty {
        match ty {
            Ty::Mono(var) => {
                if let Some(substituted_ty) = self.mappings.get(&var) {
                    substituted_ty.clone()
                } else {
                    Ty::Mono(var)
                }
            }
            Ty::Arrow(left, right) => {
                Ty::Arrow(
                    Box::new(self.apply(*left)),
                    Box::new(self.apply(*right)),
                )
            }
            _ => ty,
        }
    }
}