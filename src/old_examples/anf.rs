use std::io::Write;
use crate::backend::emit_imp;
use crate::core::anf::{CExpr, Expr};
use crate::core::common::{Atom, OpType};
use crate::core::ty::{Type, TypedIdent};

pub fn compile(name: &str) {
    let anf: Expr;
    
    if name == "factorial" {
        anf = get_factorial_anf();
    } else if name == "simple_1" {
        anf = get_simple_1();
    } else if name == "closure_return" {
        anf = get_closure_return()
    } else {
        println!("Unknown example: {}", name);
        return;
    }
    
    println!("Original ANF for example {}:", name);
    println!("{}\n", anf);

    let closure_form = anf.anf2closure().unwrap();
    
    println!("Closure form for example {}:", name);
    println!("{}\n", closure_form);

    let result = emit_imp::emit_imp(closure_form);

    // Emit the imp code to `old_examples/test.c`
    // using write
    let mut file = std::fs::File::create(format!("old_examples/{}.c", name)).unwrap();
    file.write_all(result.as_bytes()).unwrap();
    
    // call GCC to compile the C code
    let output = std::process::Command::new("gcc")
        .arg(format!("old_examples/{}.c", name))
        .arg("-o")
        .arg(format!("old_examples/{}", name))
        .output()
        .expect("Failed to compile C code");
    
    if !output.status.success() {
        eprintln!("Error compiling C code: {}", String::from_utf8_lossy(&output.stderr));
        return;
    }
}

pub fn get_factorial_anf() -> Expr {
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

    let factorial_ast = CExpr::LetFun {
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
    
    Expr::CExpr(factorial_ast)
}

fn get_simple_1() -> Expr {
    let core = CExpr::LetFun {
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
                                name: "z".to_string(),
                                ty: Type::Int,
                            }
                        ),
                    ],
                    ret_ty: Type::Int,
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
            Expr::CExpr(core)
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
            y_surround_core
        ),
    };
    
    z
}

fn get_closure_return() -> Expr {
    let g = CExpr::LetFun {
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

    let core = CExpr::LetFun {
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
            Expr::CExpr(g)
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
            Expr::CExpr(core)
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
    
    input
}