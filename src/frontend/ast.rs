use crate::frontend::FrontendError;
use crate::frontend::ty::{MonoContext, Ty};

#[derive(Debug)]
pub enum ASTExpr<T> {
    Atom(ASTAtom),
    If {
        cond: Box<ASTExpr<T>>,
        then: Box<ASTExpr<T>>,
        else_: Box<ASTExpr<T>>,
        ty: T,
    },
    Let {
        bind: (String, T),
        value: Box<ASTExpr<T>>,
        body: Box<ASTExpr<T>>,
        ty: T,
    },
    /// Single argument function application
    Apply {
        func: Box<ASTExpr<T>>,
        args: Box<ASTExpr<T>>,
        ty: T,
    },
    /// Single argument lambda expression
    Lambda {
        arg: (String, T),
        body: Box<ASTExpr<T>>,
        ret_ty: T,
    },
}

pub type InputASTExpr = ASTExpr<Option<Ty>>;

#[derive(Debug)]
pub enum ASTAtom {
    Int(i32),
    Var(String),
    Op(OpType),
}

impl ASTAtom {
    pub fn ty(&self, gamma: &mut MonoContext) -> Result<Ty, FrontendError> {
        match self {
            ASTAtom::Int(_) => Ok(Ty::Int),
            ASTAtom::Var(var) => {
                if let Some(ty) = gamma.lookup(var).cloned() {
                    Ok(ty)
                } else {
                    Err(FrontendError::UnboundVariable(var.clone()))
                }
            }
            ASTAtom::Op(op) => match op {
                OpType::Add | OpType::Sub | OpType::Mul | OpType::Div => Ok(
                    Ty::Arrow(Box::new(Ty::Int), Box::new(Ty::Arrow(
                        Box::new(Ty::Int),
                        Box::new(Ty::Int),
                    )))
                ),
                OpType::Eq | OpType::Neq | OpType::Lt | OpType::Gt | OpType::Leq | OpType::Geq => Ok(
                    Ty::Arrow(Box::new(Ty::Int), Box::new(Ty::Arrow(
                        Box::new(Ty::Int),
                        Box::new(Ty::Bool),
                    )))
                ),
            },
        }
    }
}

#[derive(Debug)]
pub enum OpType {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
}

#[macro_export]
macro_rules! curry_binop {
    ($op:expr, $l:expr, $r:expr) => {
        InputASTExpr::Apply {
            func: Box::new(InputASTExpr::Apply {
                func: Box::new(InputASTExpr::Atom(ASTAtom::Op($op))),
                args: Box::new($l),
                ty: None,
            }),
            args: Box::new($r),
            ty: None, // Type can be inferred later
        }
    };
}