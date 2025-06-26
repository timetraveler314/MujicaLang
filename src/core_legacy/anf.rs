use std::collections::HashSet;
use crate::core::common::{Atom, OpType};
use crate::core::ty::{Type, TypedIdent};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Closure {
    pub global_name: String,
    pub ret_ty: Type,
    pub capture: Vec<TypedIdent>,
    pub args: Vec<TypedIdent>,
}

#[derive(Debug)]
pub enum CExpr {
    Atom(Atom),
    Op {
        op: OpType,
        args: Vec<Atom>,
    },
    Call {
        closure: Atom,
        args: Vec<Atom>,
        ret_ty: Type,
    },
    If {
        cond: Box<Atom>,
        then: Box<Expr>,
        else_: Box<Expr>,
        ty: Type,
    },
    LetFun {
        bind: TypedIdent,
        args: Vec<TypedIdent>,
        body: Box<Expr>,
        body2: Box<Expr>,
    },
    LetClos {
        bind: TypedIdent,
        body: Box<Expr>,
    }
}

#[derive(Debug)]
pub enum Expr {
    CExpr(CExpr),
    Let {
        bind: TypedIdent,
        value: Box<CExpr>,
        body: Box<Expr>,
    },
}

impl Atom {
    pub fn free_vars(&self) -> HashSet<TypedIdent> {
        match self {
            Atom::Int(_) => HashSet::new(),
            Atom::Var(var) => {
                let mut set = HashSet::new();
                set.insert(var.clone());
                set
            }
            _ => HashSet::new(),
        }
    }
    
    pub fn ty(&self) -> Type {
        match self {
            Atom::Int(_) => Type::Int,
            Atom::Var(var) => var.ty.clone(),
            Atom::InputInt => Type::Int,
        }
    }
}

impl CExpr {
    fn free_vars_func_body(&self) -> HashSet<TypedIdent> {
        if let CExpr::LetFun { bind, args, body, .. } = self {
            let mut body_vars = body.free_vars();
            // Arguments are not free in the function body
            body_vars.retain(|v| !args.contains(v));
            // The function name is not free in the function body
            body_vars.retain(|v| v.name != bind.name);

            return body_vars;
        }

        HashSet::new()
    }
    
    pub fn free_vars(&self) -> HashSet<TypedIdent> {
        match self {
            CExpr::Atom(atom) => atom.free_vars(),
            CExpr::Op { args, .. } => {
                let mut vars = HashSet::new();
                for arg in args {
                    vars.extend(arg.free_vars());
                }
                vars
            }
            CExpr::Call { closure, args, ret_ty: _ret_ty } => {
                let mut vars = closure.free_vars();
                for arg in args {
                    vars.extend(arg.free_vars());
                }
                vars
            }
            CExpr::If { cond, then, else_, .. } => {
                let mut vars = cond.free_vars();
                vars.extend(then.free_vars());
                vars.extend(else_.free_vars());
                vars
            }
            CExpr::LetFun { bind, body2, .. } => {
                let mut vars = self.free_vars_func_body();
                let let_body_vars = body2.free_vars();

                vars.extend(let_body_vars);

                // Function name is not free in the `let` body
                vars.retain(|v| v.name != bind.name);
                vars
            }
            CExpr::LetClos { bind, body, .. } => {
                let mut vars = body.free_vars();
                vars.retain(|v| v.name != bind.name);
                vars
            }
        }
    }
}

impl Expr { 
    pub fn free_vars(&self) -> HashSet<TypedIdent> {
        match self {
            Expr::CExpr(cexp) => cexp.free_vars(),
            Expr::Let { bind, value, body } => {
                let mut vars = value.free_vars();
                let mut body_vars = body.free_vars();
                body_vars.retain(|v| v.name != bind.name);
                vars.extend(body_vars);
                vars
            }
        }
    }
}