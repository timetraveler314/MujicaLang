use crate::frontend::hm::subst::Subst;
use crate::frontend::hm::TypeError;
use crate::frontend::ty::{Ty, TypeVar};

#[derive(Debug)]
#[derive(Clone)]
pub enum Constraint {
    Eq(Ty, Ty), // Represents an equality constraint between two types
}

impl Constraint {
    pub fn apply(&self, var: &TypeVar, ty: &Ty) -> Constraint {
        match self {
            Constraint::Eq(t1, t2) => {
                let t1 = t1.apply(var, ty);
                let t2 = t2.apply(var, ty);
                Constraint::Eq(t1, t2)
            }
        }
    }
}
