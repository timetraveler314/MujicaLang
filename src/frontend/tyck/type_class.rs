use std::fmt;
use std::fmt::Display;

/// The structure representing a type class constraint in the type system.
/// `Class type_var` indicates that the `type_var` must be an instance of the `class`.
#[derive(Debug, Clone)]
pub struct TypeClassConstraint {
    pub class: String,
    pub type_var: String,
}

impl Display for TypeClassConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.class, self.type_var)
    }
}
