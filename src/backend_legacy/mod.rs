pub(crate) mod closure_conversion;
mod imp_builder;
mod imp;
pub(crate) mod emit_imp;

#[derive(Debug)]
#[allow(unused)]
pub enum Error {
    UnboundVariable(String),
    TypeError(String),
    NonClosureError(String),
    UnboundFunction(String),
}