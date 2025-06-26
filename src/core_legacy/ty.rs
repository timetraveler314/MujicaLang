use std::rc::Rc;
use crate::core::anf::Closure;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    Int,
    Unit,
    Function(Vec<Type>, Box<Type>),
    Closure(Rc<Closure>),
    Bool,
}

impl Type {
    pub fn from_ident(ident: &str) -> Self {
        match ident.as_ref() {
            "int" => Type::Int,
            "unit" => Type::Unit,
            "bool" => Type::Bool,
            _ => panic!("Unknown type identifier: {}", ident),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypedIdent {
    pub name: String,
    pub ty: Type
}