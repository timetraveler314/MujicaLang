#[derive(Debug)]
pub enum Ty {
    Unit,
    Int,
    Bool,
    Arrow(Box<Ty>, Box<Ty>),
    
    /// A monomorphic type, or a type variable
    Mono(String),
    /// A parametric type
    Poly(String, Box<Ty>),
}
