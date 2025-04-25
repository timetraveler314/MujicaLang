use std::io::Write;
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
                                name: "z".to_string(),
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
            CExpr::If {
                cond: Box::new(
                    Atom::Var(
                        TypedIdent {
                            name: "input_eq_0".to_string(),
                            ty: Type::Int,
                        }
                    )
                ),
                then: Box::new(
                    Expr::CExpr(
                        CExpr::Atom(
                            Atom::Int(1)
                        )
                    )
                ),
                else_: Box::new(
                    Expr::CExpr(
                        CExpr::Atom(
                            Atom::Int(2)
                        )
                    )
                ),
                ty: Type::Int,
            }
        ),
        body: Box::new(
            y_surround
        ),
    };
    
    let input_eq_0 = Expr::Let {
        bind: TypedIdent { name: "input_eq_0".to_string(), ty: Type::Int },
        value: Box::new(
            CExpr::Op {
                op: OpType::Eq,
                args: vec![
                    Atom::Var(
                        TypedIdent {
                            name: "input".to_string(),
                            ty: Type::Int,
                        }
                    ),
                    Atom::Int(0),
                ],
            }
        ),
        body: Box::new(
            z
        ),
    };
    
    let input = Expr::Let {
        bind: TypedIdent { name: "input".to_string(), ty: Type::Int },
        value: Box::new(
            CExpr::Atom(
                Atom::InputInt
            )
        ),
        body: Box::new(
            input_eq_0
        ),
    };

    let anf = input;
    
    println!("Original ANF:");
    println!("{}", anf);

    let closure_form = anf.anf2closure().unwrap();

    println!("{}", closure_form);

    let result = emit_imp::emit_imp(closure_form);
    
    // Emit the imp code to `t/test.c`
    // using write
    let mut file = std::fs::File::create("t/test.c").unwrap();
    file.write_all(result.as_bytes()).unwrap();
}
