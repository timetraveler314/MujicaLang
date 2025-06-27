use std::collections::HashMap;
use crate::frontend::ty::{Ty, TypeVar};

pub fn apply_subst(ty: Ty, subst: HashMap<TypeVar, Ty>) -> Ty {
    match ty {
        Ty::Mono(var) => {
            if let Some(substituted_ty) = subst.get(&var) {
                substituted_ty.clone()
            } else {
                Ty::Mono(var)
            }
        }
        Ty::Arrow(left, right) => {
            Ty::Arrow(
                Box::new(apply_subst(*left, subst.clone())),
                Box::new(apply_subst(*right, subst)),
            )
        }
        _ => ty,
    }
}