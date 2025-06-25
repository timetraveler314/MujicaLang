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
        ASTExpr::Atom(atom, ty) => {
            let atom_str = pretty_atom(atom);
            match ty {
                Some(t) => format!("{}{} : {}", pad, atom_str, t),
                None => format!("{}{}", pad, atom_str),
            }
        }
        ASTExpr::If { cond, then, else_, ty } => {
            let cond_str = pretty_expr(cond, indent + 1);
            let then_str = pretty_expr(then, indent + 1);
            let else_str = pretty_expr(else_, indent + 1);
            let type_anno = ty.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default();

            format!(
                "{pad}if {}\n{pad}then {}\n{pad}else {}\n{pad}end{}",
                cond_str, then_str, else_str, type_anno,
                pad = pad
            )
        }
        ASTExpr::Let { bind: (name, scheme), value, body, ty } => {
            let name_str = match scheme {
                Some(scheme) => format!("{} : {}", name, scheme),
                None => name.to_string(),
            };

            let value_str = pretty_expr(value, indent + 1);
            let body_str = pretty_expr(body, indent + 1);
            let type_anno = ty.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default();

            format!(
                "{pad}let {name_str} =\n{}\n{pad}in\n{}\n{pad}end{}",
                value_str, body_str, type_anno,
                pad = pad
            )
        }
        ASTExpr::Apply { func, args, ty } => {
            let func_str = pretty_expr(func, 0);
            let args_str = pretty_expr(args, 0);
            let type_anno = ty.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default();

            format!(
                "{}({} {}){}",
                pad, func_str, args_str, type_anno
            )
        }
        ASTExpr::Lambda { arg: (name, ty), body, ret_ty } => {
            let arg_type = ty.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default();
            let body_str = pretty_expr(body, indent + 1);
            let ret_type = ret_ty.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default();

            format!(
                "{pad}fun {}{} ->\n{}{}",
                name, arg_type, body_str, ret_type,
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