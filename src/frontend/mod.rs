use lalrpop_util::lalrpop_mod;
use crate::frontend::hm::TypeError;

#[macro_use] pub(super) mod ast;
mod ty;
pub(crate) mod hm;

lalrpop_mod!(mujicalang, "/frontend/mujicalang.rs");

#[derive(Debug)]
#[allow(dead_code)]
pub enum FrontendError {
    TypeError(TypeError),
    ParseError(String),
    UnboundVariable(String),
}

pub fn parse(input: &str) -> ast::InputASTExpr {
    mujicalang::ExprParser::new().parse(input).unwrap()
}