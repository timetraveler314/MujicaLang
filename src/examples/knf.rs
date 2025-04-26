use crate::core::common::{Atom, OpType};
use crate::core::knf;
use crate::core::ty::{Type, TypedIdent};
pub fn get_simple() -> knf::Expr {
    let let_z_complex = knf::Expr::Let {
        bind: TypedIdent {
            name: "z".to_string(),
            ty: Type::Int,
        },
        value: Box::new(knf::Expr::Let {
            // let w = x - 1 in w + y end
            bind: TypedIdent {
                name: "w".to_string(),
                ty: Type::Int,
            },
            value: Box::new(knf::Expr::Op {
                op: OpType::Sub,
                args: vec![
                    Atom::Var(TypedIdent {
                        name: "x".to_string(),
                        ty: Type::Int,
                    }),
                    Atom::Int(1),
                ],
            }),
            body: Box::new(knf::Expr::Op {
                op: OpType::Add,
                args: vec![
                    Atom::Var(TypedIdent {
                        name: "w".to_string(),
                        ty: Type::Int,
                    }),
                    Atom::Var(TypedIdent {
                        name: "y".to_string(),
                        ty: Type::Int,
                    }),
                ],
            })
        }),
        body: Box::new(
            // x - z
            knf::Expr::Op {
                op: OpType::Sub,
                args: vec![
                    Atom::Var(TypedIdent {
                        name: "x".to_string(),
                        ty: Type::Int,
                    }),
                    Atom::Var(TypedIdent {
                        name: "z".to_string(),
                        ty: Type::Int,
                    }),
                ],
            }
        ),
    };

    let let_y_2 = knf::Expr::Let {
        bind: TypedIdent {
            name: "y".to_string(),
            ty: Type::Int,
        },
        value: Box::new(knf::Expr::Atom(Atom::Int(2))),
        body: Box::new(let_z_complex),
    };

    let knf: knf::Expr = knf::Expr::Let {
        bind: TypedIdent {
            name: "x".to_string(),
            ty: Type::Int,
        },
        value: Box::new(knf::Expr::Atom(Atom::Int(42))),
        body: Box::new(let_y_2),
    };
    
    knf
}