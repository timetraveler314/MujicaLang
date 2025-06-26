use crate::core::ty::TypedIdent;

#[derive(Debug, Clone)]
pub enum Atom {
    Int(i32),
    Var(TypedIdent),
    InputInt,
}

#[derive(Debug)]
pub enum OpType {
    Add,
    Eq,
    Sub,
    Mul,
    Div,
}
