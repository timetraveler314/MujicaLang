use lalrpop_util::lalrpop_mod;

#[macro_use] pub(super) mod ast;
mod ty;
mod hm;

lalrpop_mod!(mujicalang, "/frontend/mujicalang.rs");

#[derive(Debug)]
#[allow(dead_code)]
pub enum FrontendError {
    TypeError(String),
    ParseError(String),
    UnboundVariable(String),
}

pub fn parse(input: &str) -> ast::ASTExpr {
    mujicalang::ExprParser::new().parse(input).unwrap()
}