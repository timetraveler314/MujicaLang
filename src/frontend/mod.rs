use lalrpop_util::lalrpop_mod;

pub(super) mod ast;

lalrpop_mod!(mujicalang, "/frontend/mujicalang.rs");

pub fn parse(input: &str) -> ast::ASTExpr {
    mujicalang::ExprParser::new().parse(input).unwrap()
}