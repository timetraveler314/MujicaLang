use crate::backend::emit_imp;
use crate::core::anf::{Atom, CExpr, Expr, OpType};
use crate::core::ty::{Type, TypedIdent};

mod core;
mod backend;
mod util;

fn main() {

    let g = Expr::LetFun {
        bind: TypedIdent {
            name: "g".to_string(),
            ty: Type::Function(
                vec![Type::Int],
                Box::new(Type::Function(
                    vec![Type::Int],
                    Box::new(Type::Int),
                ),),
            ),
        },
        args: vec![
            TypedIdent {
                name: "gx".to_string(),
                ty: Type::Int,
            },
        ],
        body: Box::new(
            Expr::CExpr(
                CExpr::Atom(
                    Atom::Var(TypedIdent {
                        name: "f".to_string(),
                        ty: Type::Function(
                            vec![Type::Int],
                            Box::new(Type::Int),
                        ),
                    })
                )
            )
        ),
        body2: Box::new(
            Expr::Let {
                bind: TypedIdent {
                    name: "g_applied".to_string(),
                    ty: Type::Function(
                        vec![Type::Int],
                        Box::new(Type::Int),
                    )
                },
                value: Box::new(
                        CExpr::Call {
                            closure: Atom::Var(
                                TypedIdent {
                                    name: "g".to_string(),
                                    ty: Type::Function(
                                        vec![Type::Int],
                                        Box::new(Type::Function(
                                            vec![Type::Int],
                                            Box::new(Type::Int),
                                        ),))
                                }
                            ),
                            args: vec![
                                Atom::Var(
                                    TypedIdent {
                                        name: "y".to_string(),
                                        ty: Type::Int,
                                    }
                                ),
                            ],
                            ret_ty: Type::Function(
                                vec![Type::Int],
                                Box::new(Type::Int),
                            ),
                        }
                ),
                body: Box::new(
                    Expr::CExpr(
                        CExpr::Call {
                            closure: Atom::Var(
                                TypedIdent {
                                    name: "g_applied".to_string(),
                                    ty: Type::Function(
                                        vec![Type::Int],
                                        Box::new(Type::Int),
                                    ),
                                }
                            ),
                            args: vec![
                                Atom::Var(
                                    TypedIdent {
                                        name: "y".to_string(),
                                        ty: Type::Int,
                                    }
                                ),
                            ],
                            ret_ty: Type::Int,
                        }
                    )
                ),
            }
        ),
    };
    
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
            g
        )
    };

    let y_surround = Expr::Let {
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
    
    let z = Expr::Let {
        bind: TypedIdent {
            name: "z".to_string(),
            ty: Type::Int,
        },
        value: Box::new(
            CExpr::Atom(
                Atom::Int(2)
            )
        ),
        body: Box::new(
            y_surround
        ),
    };

    let anf = z;
    
    println!("Original ANF:");
    println!("{}", anf);

    let closure_form = anf.anf2closure().unwrap();

    println!("{}", closure_form);

    let result = emit_imp::emit_imp(closure_form);
    
    println!("{}", result);
}
