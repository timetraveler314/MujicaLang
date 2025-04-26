use crate::backend::emit_imp;
use crate::core::ast::Expr::{Atom, Call, If, LetFun, Op};
use crate::core::common::Atom::{InputInt, Int, Var};
use crate::core::common::OpType;
use crate::core::conversion::ast2knf::ast2knf;
use crate::core::conversion::knf2anf::knf2anf;
use crate::core::ty::{Type, TypedIdent};

pub fn fun_fact() -> String {
    let ast = LetFun {
        bind: TypedIdent {
            name: "fact".to_string(),
            ty: Type::Function(
                vec![Type::Int], Box::from(Type::Int),
            )
        },
        args: vec![TypedIdent {
            name: "n".to_string(),
            ty: Type::Int,
        }],
        body: Box::new(If {
            cond: Box::new(Op {
                op: OpType::Eq,
                args: vec![
                    Atom(Var(TypedIdent {
                        name: "n".to_string(),
                        ty: Type::Int,
                    })),
                    Atom(Int(0)),
                ],
            }),
            then: Box::new(Atom(Int(1))),
            else_: Box::new(Op {
                op: OpType::Mul,
                args: vec![
                    Atom(Var(TypedIdent {
                        name: "n".to_string(),
                        ty: Type::Int,
                    })),
                    Call {
                        closure: Box::new(Atom(Var(TypedIdent {
                            name: "fact".to_string(),
                            ty: Type::Function(
                                vec![Type::Int], Box::from(Type::Int),
                            )
                        }))),
                        args: vec![Op {
                            op: OpType::Sub,
                            args: vec![
                                Atom(Var(TypedIdent {
                                    name: "n".to_string(),
                                    ty: Type::Int,
                                })),
                                Atom(Int(1)),
                            ],
                        }],
                        ret_ty: Type::Int,
                    },
                ],
            }),
            ty: Type::Int,
        }),
        body2: Box::new(Call {
            closure: Box::new(Atom(Var(TypedIdent {
                name: "fact".to_string(),
                ty: Type::Function(
                    vec![Type::Int], Box::from(Type::Int),
                )
            }))),
            args: vec![Atom(InputInt)],
            ret_ty: Type::Int,
        }),
    };

    let knf = ast2knf(ast);
    let anf = knf2anf(knf).unwrap();
    let closure = anf.anf2closure().unwrap();
    let imp = emit_imp::emit_imp(closure);

    imp
}