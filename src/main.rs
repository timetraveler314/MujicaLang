use std::io::Write;
use crate::backend::emit_imp;
use crate::core::anf::{Atom, CExpr, Expr, OpType};
use crate::core::ty::{Type, TypedIdent};

mod core;
mod backend;
mod util;

fn main() {
    let tmp = CExpr::If {
        cond: Box::new(
            Atom::Var(
                TypedIdent {
                    name: "is_base_case".to_string(),
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
            Expr::Let {
                bind: TypedIdent {
                    name: "n_minus_1".to_string(),
                    ty: Type::Int,
                },
                value: Box::new(
                    CExpr::Op {
                        op: OpType::Sub,
                        args: vec![
                            Atom::Var(
                                TypedIdent {
                                    name: "n".to_string(),
                                    ty: Type::Int,
                                }
                            ),
                            Atom::Int(1),
                        ],
                    }
                ),
                body: Box::new(
                    Expr::Let {
                        bind: TypedIdent {
                            name: "recursive_result".to_string(),
                            ty: Type::Int,
                        },
                        value: Box::new(
                            CExpr::Call {
                                closure: Atom::Var(
                                    TypedIdent {
                                        name: "fact".to_string(),
                                        ty: Type::Function(
                                            vec![Type::Int],
                                            Box::new(Type::Int),
                                        ),
                                    }
                                ),
                                args: vec![
                                    Atom::Var(
                                        TypedIdent {
                                            name: "n_minus_1".to_string(),
                                            ty: Type::Int,
                                        }
                                    ),
                                ],
                                ret_ty: Type::Int,
                            }
                        ),
                        body: Box::new(
                            Expr::CExpr(
                                CExpr::Op {
                                    op: OpType::Mul,
                                    args: vec![
                                        Atom::Var(
                                            TypedIdent {
                                                name: "n".to_string(),
                                                ty: Type::Int,
                                            }
                                        ),
                                        Atom::Var(
                                            TypedIdent {
                                                name: "recursive_result".to_string(),
                                                ty: Type::Int,
                                            }
                                        ),
                                    ],
                                }
                            )
                        ),
                    }
                ),
            }
        ),
        ty: Type::Int,
    };
    
    let factorial_ast = Expr::LetFun {
        bind: TypedIdent {
            name: "fact".to_string(),
            ty: Type::Function(
                vec![Type::Int],
                Box::new(Type::Int),
            ),
        },
        args: vec![
            TypedIdent {
                name: "n".to_string(),
                ty: Type::Int,
            },
        ],
        body: Box::new(
            Expr::Let {
                bind: TypedIdent {
                    name: "is_base_case".to_string(),
                    ty: Type::Int,
                },
                value: Box::new(
                    CExpr::Op {
                        op: OpType::Eq,
                        args: vec![
                            Atom::Var(
                                TypedIdent {
                                    name: "n".to_string(),
                                    ty: Type::Int,
                                }
                            ),
                            Atom::Int(1),
                        ],
                    }
                ),
                body: Box::new(
                    Expr::CExpr(
                        tmp
                    )
                ),
            }
        ),
        body2: Box::new(
            Expr::Let {
                bind: TypedIdent {
                    name: "input".to_string(),
                    ty: Type::Int,
                },
                value: Box::new(
                    CExpr::Atom(
                        Atom::InputInt
                    )
                ),
                body: Box::new(
                    Expr::CExpr(
                        CExpr::Call {
                            closure: Atom::Var(
                                TypedIdent {
                                    name: "fact".to_string(),
                                    ty: Type::Function(
                                        vec![Type::Int],
                                        Box::new(Type::Int),
                                    ),
                                }
                            ),
                            args: vec![
                                Atom::Var(
                                    TypedIdent {
                                        name: "input".to_string(),
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

    let anf = factorial_ast;
    
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
