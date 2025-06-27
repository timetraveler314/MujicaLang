pub mod closure;
pub mod closure_conversion;
pub mod imp;
pub mod imp_builder;
pub mod emit_imp;

#[derive(Debug)]
pub enum BackendError {
    ImpError(String),
}