pub mod infer;
pub mod constraint;
pub mod subst;

#[derive(Debug)]
pub enum TypeError {
    UnificationError(String), // Represents an error during type unification
    ConstraintError(String),  // Represents an error in constraints
    Other(String),            // Represents any other type error
}