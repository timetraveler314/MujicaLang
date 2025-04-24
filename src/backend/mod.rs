pub(crate) mod closure_conversion;

#[derive(Debug)]
#[allow(unused)]
pub enum Error {
    UnboundVariable(String),
    TypeError(String),
}