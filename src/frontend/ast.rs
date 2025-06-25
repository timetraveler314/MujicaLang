use crate::frontend::FrontendError;
use crate::frontend::ty::{Scheme, Ty};

#[derive(Debug)]
pub enum ASTExpr<I, T, S = Option<Scheme>> {
    Atom(ASTAtom<I>, T),
    If {
        cond: Box<ASTExpr<I, T>>,
        then: Box<ASTExpr<I, T>>,
        else_: Box<ASTExpr<I, T>>,
        ty: T,
    },
    Let {
        bind: (I, S),
        value: Box<ASTExpr<I, T>>,
        body: Box<ASTExpr<I, T>>,
        ty: T,
    },
    /// Single argument function application
    Apply {
        func: Box<ASTExpr<I, T>>,
        args: Box<ASTExpr<I, T>>,
        ty: T,
    },
    /// Single argument lambda expression
    Lambda {
        arg: (I, T),
        body: Box<ASTExpr<I, T>>,
        ret_ty: T,
    },
}

// Use `String` as identifier type and `Option<Ty>` for type annotations
pub type InputASTExpr = ASTExpr<String, Option<Ty>>;

#[derive(Debug)]
pub enum ASTAtom<I> {
    Int(i32),
    Var(I),
    Op(OpType),
}

impl<I> ASTAtom<I> {
    // pub fn ty(&self, gamma: &mut MonoContext) -> Result<Ty, FrontendError> {
    //     match self {
    //         ASTAtom::Int(_) => Ok(Ty::Int),
    //         ASTAtom::Var(var) => {
    //             if let Some(ty) = gamma.lookup(var).cloned() {
    //                 Ok(ty)
    //             } else {
    //                 Err(FrontendError::UnboundVariable(var.clone()))
    //             }
    //         }
    //         ASTAtom::Op(op) => match op {
    //             OpType::Add | OpType::Sub | OpType::Mul | OpType::Div => Ok(
    //                 Ty::Arrow(Box::new(Ty::Int), Box::new(Ty::Arrow(
    //                     Box::new(Ty::Int),
    //                     Box::new(Ty::Int),
    //                 )))
    //             ),
    //             OpType::Eq | OpType::Neq | OpType::Lt | OpType::Gt | OpType::Leq | OpType::Geq => Ok(
    //                 Ty::Arrow(Box::new(Ty::Int), Box::new(Ty::Arrow(
    //                     Box::new(Ty::Int),
    //                     Box::new(Ty::Bool),
    //                 )))
    //             ),
    //         },
    //     }
    // }
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
                func: Box::new(InputASTExpr::Atom(ASTAtom::Op($op), None)),
                args: Box::new($l),
                ty: None,
            }),
            args: Box::new($r),
            ty: None, // Type can be inferred later
        }
    };
}