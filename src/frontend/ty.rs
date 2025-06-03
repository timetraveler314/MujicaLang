pub type TypeVar = String;

#[derive(Debug)]
pub enum Ty {
    Unit,
    Int,
    Bool,
    Arrow(Box<Ty>, Box<Ty>),

    /// A monomorphic type, or a type variable
    Mono(TypeVar),
}

/// Represents a type scheme, which is a type with a set of universally quantified type variables
#[derive(Debug)]
pub struct Scheme {
    pub vars: Vec<TypeVar>,
    pub ty: Ty,
}
