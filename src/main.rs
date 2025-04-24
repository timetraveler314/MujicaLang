use crate::core::anf::{Atom, CExpr, Expr, OpType};
use crate::core::ty::{Type, TypedIdent};

mod core;
mod backend;
mod util;

fn main() {
    let core = Expr::LetFun {
        bind: TypedIdent {
            name: "f".to_string(),
            ty: Type::Function(
                vec![Type::Int],
                Box::new(Type::Int),
            ),
        },
        args: vec![
            TypedIdent {
                name: "x".to_string(),
                ty: Type::Int,
            },
        ],
        body: Box::new(
            Expr::CExpr(
                CExpr::Op {
                    op: OpType::Add,
                    args: vec![
                        Atom::Var(
                            TypedIdent {
                                name: "x".to_string(),
                                ty: Type::Int,
                            }
                        ),
                        Atom::Var(
                            TypedIdent {
                                name: "y".to_string(),
                                ty: Type::Int,
                            }
                        ),
                    ],
                }
            )
        ),
        body2: Box::new(
            /* f x + 1 */
            Expr::CExpr(
                CExpr::Call {
                    closure: Atom::Var(
                        TypedIdent {
                            name: "f".to_string(),
                            ty: Type::Function(
                                vec![Type::Int],
                                Box::new(Type::Int),
                            ),
                        }
                    ),
                    args: vec![
                        Atom::Var(
                            TypedIdent {
                                name: "x".to_string(),
                                ty: Type::Int,
                            }
                        ),
                    ],
                }
            )
        ),
    };
    
    let y_surround_core = Expr::Let {
        bind: TypedIdent {
            name: "y".to_string(),
            ty: Type::Int,
        },
        value: Box::new(
            CExpr::Atom(
                Atom::Int(1)
            )
        ),
        body: Box::new(
            core
        ),
    };
    
    let anf = y_surround_core;
    
    let closure_form = anf.anf2closure().unwrap();
    
    println!("{}", closure_form);
}
