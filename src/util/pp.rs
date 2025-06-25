use crate::frontend::ast::{ASTExpr, ASTAtom, OpType};
use crate::frontend::ty::{Ty, Scheme};
use std::fmt::{self, Display};

/// Pretty-print an AST expression into a human-readable string
pub fn pretty_expr<I: Display + Clone, T: Display>(
    expr: &ASTExpr<I, Option<T>, Option<Scheme>>,
    indent: usize,
) -> String {
    let pad = "  ".repeat(indent);
    match expr {
        ASTExpr::Atom(atom) => format!("{}{}", pad, pretty_atom(atom)),
        ASTExpr::If { cond, then, else_, .. } => format!(
            "{pad}if {}\n{pad}then {}\n{pad}else {}\n{pad}end",
            pretty_expr(cond, indent + 1),
            pretty_expr(then, indent + 1),
            pretty_expr(else_, indent + 1),
            pad = pad
        ),
        ASTExpr::Let { bind: (name, scheme), value, body, .. } => {
            let name_str = match scheme {
                Some(scheme) => format!("{} : {}", name, scheme),
                None => name.to_string(),
            };
            
            format!(
                "{pad}let {name_str} =\n{}\n{pad}in\n{}\n{pad}end",
                pretty_expr(value, indent + 1),
                pretty_expr(body, indent + 1),
                pad = pad
            )
        }
        ASTExpr::Apply { func, args, .. } => format!(
            "{}({} {})",
            pad,
            pretty_expr(func, 0),
            pretty_expr(args, 0)
        ),
        ASTExpr::Lambda { arg: (name, _ty), body, .. } => {
            format!(
                "{pad}fun {} ->\n{}",
                name,
                pretty_expr(body, indent + 1),
                pad = pad
            )
        }
    }
}

/// Pretty-print an atom (variable, int, operator)
pub fn pretty_atom<I: Display>(atom: &ASTAtom<I>) -> String {
    match atom {
        ASTAtom::Int(i) => i.to_string(),
        ASTAtom::Var(name) => name.to_string(),
        ASTAtom::Op(op) => pretty_op(op),
    }
}

/// Pretty-print an operator
pub fn pretty_op(op: &OpType) -> String {
    use OpType::*;
    match op {
        Add => "+".to_string(),
        Sub => "-".to_string(),
        Mul => "*".to_string(),
        Div => "/".to_string(),
        Eq => "==".to_string(),
        Neq => "!=".to_string(),
        Lt => "<".to_string(),
        Gt => ">".to_string(),
        Leq => "<=".to_string(),
        Geq => ">=".to_string(),
    }
}
