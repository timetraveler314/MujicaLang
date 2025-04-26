use std::rc::Rc;
use crate::backend::Error;
use crate::core::anf::{CExpr, Closure, Expr};
use crate::core::ty::{Type, TypedIdent};

#[derive(Debug)]
pub struct GlobalFuncDef {
    pub local_name: String,
    pub clos: Rc<Closure>,
    pub body: Box<Expr>,
}

#[derive(Debug, Default)]
pub struct ClosureBuilder {
    pub functions: Vec<GlobalFuncDef>,
    pub main: Option<Expr>,
    counter: usize,
}

impl ClosureBuilder {
    pub fn next_counter(&mut self) -> usize {
        let counter = self.counter;
        self.counter += 1;
        counter
    }
}

impl Expr {
    pub fn anf2closure(self) -> Result<ClosureBuilder, Error> {
        let mut program: ClosureBuilder = Default::default();

        let transformed_main = self.transform(&mut program)?;
        program.main = Some(transformed_main);

        Ok(program)
    }
}

pub trait ANF2Closure {
    type Output;
    fn transform(self, program: &mut ClosureBuilder) -> Result<Self::Output, Error>;
}

impl ANF2Closure for Expr {
    type Output = Expr;
    fn transform(self, program: &mut ClosureBuilder) -> Result<Expr, Error> {
        match self {
            Expr::CExpr(cexp) => {
                Ok(Expr::CExpr(cexp.transform(program)?))
            }
            Expr::Let { bind, value, body } => {
                let value = value.transform(program)?;
                let body = body.transform(program)?;

                Ok(Expr::Let {
                    bind: bind.clone(),
                    value: Box::new(value),
                    body: Box::new(body),
                })
            }
        }
    }
}

impl ANF2Closure for CExpr {
    type Output = CExpr;

    fn transform(self, program: &mut ClosureBuilder) -> Result<Self::Output, Error> {
        match self {
            CExpr::LetFun { bind, args, body, body2 } => {
                let global_name = format!("closure_{}_{}", bind.name, program.next_counter());

                let body_free_vars = body.free_vars();
                let free_vars = body_free_vars.iter()
                    .filter(|v| !args.contains(v))
                    .cloned()
                    .collect::<Vec<_>>();

                let ret_ty = match bind.ty {
                    Type::Function(_args, ret) => {
                        ret.clone()
                    }
                    _ => {
                        return Err(Error::TypeError(format!(
                            "Expected function type, found: {:?}",
                            bind.ty
                        )));
                    }
                };

                let closure = Rc::new(
                    Closure {
                        global_name: global_name.clone(),
                        ret_ty: *ret_ty,
                        capture: free_vars,
                        args,
                    }
                );

                let function = GlobalFuncDef {
                    local_name: bind.name.clone(),
                    clos: closure.clone(),
                    body: Box::from(body.transform(program)?),
                };

                program.functions.push(function);

                let let_clos = CExpr::LetClos {
                    bind: TypedIdent {
                        name: bind.name,
                        ty: Type::Closure(closure)
                    },
                    body: Box::new(body2.transform(program)?),
                };

                Ok(let_clos)
            }
            CExpr::LetClos { .. } => unimplemented!(),
            _ => {
                Ok(self)
            }
        }
    }
}