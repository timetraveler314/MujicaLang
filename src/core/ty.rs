use std::rc::Rc;
use crate::core::anf::Closure;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Unit,
    Function(Vec<Type>, Box<Type>),
    Closure(Rc<Closure>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedIdent {
    pub name: String,
    pub ty: Type
}