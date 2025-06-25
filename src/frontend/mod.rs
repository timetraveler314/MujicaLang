use lalrpop_util::lalrpop_mod;

#[macro_use] pub(super) mod ast;
pub mod ty;
// pub(crate) mod hm;
pub mod name_resolution;
pub mod tyck;

lalrpop_mod!(mujicalang, "/frontend/mujicalang.rs");

#[derive(Debug)]
#[allow(dead_code)]
pub enum FrontendError {
    TypeError(String),
    ParseError(String),
    UnboundVariable(String),
}

pub fn parse(input: &str) -> ast::InputASTExpr {
    mujicalang::ExprParser::new().parse(input).unwrap()
}