use lalrpop_util::lalrpop_mod;

pub(super) mod ast;
mod type_check;

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