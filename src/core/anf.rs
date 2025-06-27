use std::collections::HashSet;
use crate::core::{Atom, TypedAtom};
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;
use crate::util::pp::pretty_atom;

#[derive(Debug, Clone)]
pub enum CExpr {
    Atom(TypedAtom),
    If {
        cond: Atom,
        then: Box<Expr>,
        else_: Box<Expr>,
        ty: Ty,
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

#[derive(Debug, Clone)]
pub enum Expr {
    CExpr(CExpr),
    Let {
        bind: ResolvedIdent,
        value: Box<CExpr>,
        body: Box<Expr>,
        ty: Ty,
        is_polymorphic: bool,
    },
}

impl Expr {
    pub fn free_vars(&self) -> HashSet<(ResolvedIdent, Ty)> {
        match self {
            Expr::CExpr(cexpr) => cexpr.free_vars(),
            Expr::Let { bind, value, body, .. } => {
                let mut vars = value.free_vars();
                vars.extend(body.free_vars());
                vars.retain(|(id, _)| id != bind);
                vars
            },
        }
    }
}

impl CExpr {
    pub fn free_vars(&self) -> HashSet<(ResolvedIdent, Ty)> {
        match self {
            CExpr::Atom(typed_atom) => typed_atom.free_vars(),
            CExpr::If { cond, then, else_, .. } => {
                let mut vars = TypedAtom {
                    atom: cond.clone(),
                    ty: Ty::Bool
                }.free_vars();
                
                vars.extend(then.free_vars());
                vars.extend(else_.free_vars());
                vars
            },
            CExpr::Apply { func, args, .. } => {
                let mut vars = func.free_vars();
                
                for arg in args {
                    vars.extend(arg.free_vars());
                }
                
                vars
            },
            CExpr::Lambda { args, body, .. } => {
                let mut vars = body.free_vars();
                
                println!("Vars before removing args: {:?}", vars);
                
                for (id, _ty) in args {
                    vars.retain(|(var_id, _)| var_id != id);
                }
                vars
            },
        }
    }
}

impl TypedAtom {
    pub fn free_vars(&self) -> HashSet<(ResolvedIdent, Ty)> {
        match &self.atom {
            Atom::Var(var) => {
                let mut result = HashSet::new();
                result.insert((var.clone(), self.ty.clone()));
                result
            }
            _ => HashSet::new()
        }
    }
}

impl CExpr {
    pub fn pretty(&self, indent: usize) -> String {
        match self {
            CExpr::Atom(typed_atom) => pretty_atom(&typed_atom.atom),
            CExpr::If { cond, then, else_, ty: _ } => {
                let then_str = Expr::pretty_with_indent(then, indent + 2);
                let else_str = Expr::pretty_with_indent(else_, indent + 2);
                format!(
                    "if {}\n{}then {}\n{}else {}",
                    pretty_atom(cond),
                    spaces(indent + 2),
                    then_str,
                    spaces(indent + 2),
                    else_str
                )
            },
            CExpr::Apply { func, args, ty: _ } => {
                let mut parts = vec![pretty_atom(&func.atom)];
                parts.extend(args.iter().map(|arg| pretty_atom(&arg.atom)));
                parts.join(" ")
            },
            CExpr::Lambda { args, body, ret_ty: _ } => {
                let args_str = args.iter()
                    .map(|(id, ty)| format!("({}: {})", id, ty))
                    .collect::<Vec<_>>()
                    .join(" ");
                let body_str = Expr::pretty_with_indent(body, indent + 2);
                format!(
                    "fun {} ->\n{}{}",
                    args_str,
                    spaces(indent + 2),
                    body_str
                )
            },
        }
    }
}

impl Expr {
    pub fn pretty(&self) -> String {
        Self::pretty_with_indent(self, 0)
    }

    fn pretty_with_indent(expr: &Expr, indent: usize) -> String {
        match expr {
            Expr::CExpr(cexpr) => cexpr.pretty(indent),
            Expr::Let { bind, value, body, ty: _, is_polymorphic } => {
                let value_str = CExpr::pretty(value, indent + 2);
                let body_str = Self::pretty_with_indent(body, indent + 2);
                let poly_marker = if *is_polymorphic { "poly " } else { "" };
                format!(
                    "let {}{} = {}\n{}in {}",
                    poly_marker,
                    bind,
                    value_str,
                    spaces(indent),
                    body_str
                )
            },
        }
    }
}

fn spaces(n: usize) -> String {
    " ".repeat(n)
}