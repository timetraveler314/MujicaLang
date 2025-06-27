use crate::frontend::ast::{ASTExpr, ASTAtom, OpType};
use crate::frontend::ty::{Ty, Scheme};
use std::fmt::{Display};

/// Trait for types that can be displayed as type annotations
pub trait TypeDisplay {
    fn format_type(&self) -> String;
}

impl TypeDisplay for Ty {
    fn format_type(&self) -> String {
        format!(" : {}", self)
    }
}

impl TypeDisplay for Option<Ty> {
    fn format_type(&self) -> String {
        self.as_ref().map(|t| format!(" : {}", t)).unwrap_or_default()
    }
}

impl TypeDisplay for Scheme {
    fn format_type(&self) -> String {
        format!(" : {}", self)
    }
}

impl TypeDisplay for Option<Scheme> {
    fn format_type(&self) -> String {
        self.as_ref().map(|s| format!(" : {}", s)).unwrap_or_default()
    }
}

/// Pretty-print an AST expression into a human-readable string
pub fn pretty_expr<I: Display + Clone, T: TypeDisplay>(
    expr: &ASTExpr<I, T, Option<Scheme>>,
    indent: usize,
) -> String {
    let pad = "  ".repeat(indent);
    match expr {
        ASTExpr::Atom(atom, ty) => {
            let atom_str = pretty_atom(atom);
            format!("{}{}{}", pad, atom_str, ty.format_type())
        }
        ASTExpr::If { cond, then, else_, ty } => {
            let cond_str = pretty_expr(cond, indent + 1);
            let then_str = pretty_expr(then, indent + 1);
            let else_str = pretty_expr(else_, indent + 1);

            format!(
                "{pad}if {}\n{pad}then {}\n{pad}else {}\n{pad}end{}",
                cond_str, then_str, else_str, ty.format_type(),
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

            format!(
                "{pad}let {name_str} =\n{}\n{pad}in\n{}\n{pad}end{}",
                value_str, body_str, ty.format_type(),
                pad = pad
            )
        }
        ASTExpr::Apply { func, args, ty } => {
            let func_str = pretty_expr(func, 0);
            let args_str = pretty_expr(args, 0);

            format!(
                "{}({} {}){}",
                pad, func_str, args_str, ty.format_type()
            )
        }
        ASTExpr::Lambda { arg: (name, ty), body, ret_ty } => {
            let arg_type = ty.format_type();
            let body_str = pretty_expr(body, indent + 1);

            format!(
                "{pad}fun {}{} ->\n{}{}",
                name, arg_type, body_str, ret_ty.format_type(),
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
        ASTAtom::Unit => "()".to_string(),
        ASTAtom::Bool(bool) => bool.to_string(),
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