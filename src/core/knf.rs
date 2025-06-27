use crate::core::{Atom, TypedAtom};
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;
use crate::util::pp::{pretty_atom, pretty_op};

pub enum Expr {
    Atom(TypedAtom),
    If {
        cond: Atom,
        then: Box<Expr>,
        else_: Box<Expr>,
        ty: Ty,
    },
    Let {
        bind: ResolvedIdent,
        value: Box<Expr>,
        body: Box<Expr>,
        ty: Ty,
        is_polymorphic: bool,
    },
    Apply {
        func: TypedAtom,
        args: Vec<TypedAtom>,
        ty: Ty,
    },
    Lambda {
        args: Vec<(ResolvedIdent, Ty)>,
        body: Box<Expr>,
        ret_ty: Ty,
    },
}

impl Expr {
    pub fn ty(&self) -> Ty {
        match self {
            Expr::Atom(typed_atom) => typed_atom.ty.clone(),
            Expr::If { ty, .. } => ty.clone(),
            Expr::Let { ty, .. } => ty.clone(),
            Expr::Apply { ty, .. } => ty.clone(),
            Expr::Lambda { ret_ty, .. } => ret_ty.clone(),
        }
    }
}

pub fn pretty_expr(expr: &Expr) -> String {
    pretty_expr_with_indent(expr, 0)
}

fn pretty_expr_with_indent(expr: &Expr, indent: usize) -> String {
    let next_indent = indent + 2;

    match expr {
        Expr::Atom(typed_atom) => pretty_atom(&typed_atom.atom),
        Expr::If { cond, then, else_, .. } => {
            let cond_str = pretty_atom(cond);
            let then_str = pretty_expr_with_indent(then, next_indent);
            let else_str = pretty_expr_with_indent(else_, next_indent);

            format!(
                "if {}\n{}then {}\n{}else {}\n{}end",
                cond_str,
                spaces(indent),
                then_str,
                spaces(indent),
                else_str,
                spaces(indent)
            )
        }
        Expr::Let { bind, value, body, .. } => {
            let val_str = pretty_expr_with_indent(value, next_indent);
            let body_str = pretty_expr_with_indent(body, next_indent);

            format!(
                "let {} = {}\n{}in {}\n{}end",
                bind,
                val_str,
                spaces(indent),
                body_str,
                spaces(indent)
            )
        }
        Expr::Apply { func, args, .. } => {
            let mut parts = vec![pretty_atom(&func.atom)];
            parts.extend(args.iter().map(|typed_arg| pretty_atom(&typed_arg.atom)));
            parts.join(" ")
        }
        Expr::Lambda { args, body, .. } => {
            let args_str = args
                .iter()
                .map(|(id, ty)| format!("({}: {})", id, ty))
                .collect::<Vec<_>>()
                .join(" ");
            let body_str = pretty_expr_with_indent(body, next_indent);

            format!(
                "fun {} ->\n{}{}",
                args_str,
                spaces(next_indent),
                body_str
            )
        }
    }
}

fn spaces(n: usize) -> String {
    " ".repeat(n)
}
