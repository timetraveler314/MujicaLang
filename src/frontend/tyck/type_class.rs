/// The structure representing a type class constraint in the type system.
/// `Class type_var` indicates that the `type_var` must be an instance of the `class`.
#[derive(Debug)]
pub struct TypeClassConstraint {
    pub class: String,
    pub type_var: String,
}