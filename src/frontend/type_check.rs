use crate::core::ty::{Type, TypedIdent};
use crate::core::typed_ast;
use crate::frontend::ast::{ASTAtom, ASTExpr};
use crate::frontend::FrontendError;

pub struct TypingContext {
    mapping: Vec<(String, Type)>,
}

impl TypingContext {
    pub fn new() -> Self {
        TypingContext { mapping: Vec::new() }
    }

    pub fn add(&mut self, name: String, ty: Type) {
        self.mapping.push((name, ty));
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        for (n, ty) in self.mapping.iter().rev() {
            if n == name {
                return Some(ty);
            }
        }
        None
    }
    
    pub fn pop(&mut self) {
        self.mapping.pop();
    }
}

impl ASTAtom {
    pub(self) fn to_typed_impl(
        self,
        gamma: &mut TypingContext,
    ) -> Result<(typed_ast::Atom, Type), FrontendError> {
        match self {
            ASTAtom::Int(i) => Ok((typed_ast::Atom::Int(i), Type::Int)),
            ASTAtom::Var(name) => {
                if let Some(ty) = gamma.get(&name) {
                    Ok((typed_ast::Atom::Var(
                        TypedIdent {
                            name,
                            ty: ty.clone(),
                        }), ty.clone()))
                } else {
                    Err(FrontendError::UnboundVariable(name))
                }
            }
        }
    }
}

impl ASTExpr {
    pub fn to_typed(self) -> Result<typed_ast::Expr, FrontendError> {
        let mut gamma = TypingContext::new();
        self.to_typed_impl(&mut gamma).map(|(ast, _)| ast)
    }
    
    fn to_typed_impl(
        self,
        gamma: &mut TypingContext,
    ) -> Result<(typed_ast::Expr, Type), FrontendError> {
        match self {
            ASTExpr::Atom(atom) => {
                let (atom, ty) = atom.to_typed_impl(gamma)?;
                Ok((typed_ast::Expr::Atom(atom), ty))
            }
            ASTExpr::Op { op, args } => {
                let mut typed_args: Vec<typed_ast::Expr> = Vec::new();
                let mut arg_types = Vec::new();
                for arg in args {
                    let (current, current_ty) = arg.to_typed_impl(gamma)?;
                    
                    arg_types.push(current_ty);
                    typed_args.push(current);
                }
                
                Ok((typed_ast::Expr::Op {
                    op,
                    args: typed_args,
                    ty: Type::Int, // Placeholder for actual type checking
                }, Type::Int)) // Placeholder for actual type checking
            }
            ASTExpr::If { cond, then, else_ } => {
                let (cond, cond_ty) = cond.to_typed_impl(gamma)?;
                let (then, then_ty) = then.to_typed_impl(gamma)?;
                let (else_, else_ty) = else_.to_typed_impl(gamma)?;
                
                // if cond_ty != Type::Bool {
                //     return Err(FrontendError::TypeError("Condition must be a boolean".to_string()));
                // }
                
                if then_ty != else_ty {
                    return Err(FrontendError::TypeError("Branches must have the same type".to_string()));
                }
                
                Ok((typed_ast::Expr::If {
                    cond: Box::new(cond),
                    then: Box::new(then),
                    else_: Box::new(else_),
                    ty: then_ty.clone(),
                }, then_ty))
            }
            ASTExpr::Let { bind, value, body } => {
                let (value, value_ty) = value.to_typed_impl(gamma)?;
                
                gamma.add(bind.name.clone(), value_ty.clone());
                
                let (body, body_ty) = body.to_typed_impl(gamma)?;
                
                gamma.pop();
                
                Ok((typed_ast::Expr::Let {
                    bind: TypedIdent {
                        name: bind.name,
                        ty: value_ty,
                    },
                    value: Box::new(value),
                    body: Box::new(body),
                }, body_ty))
            }
            ASTExpr::LetFun { bind, args, body, body2 } => {
                let mut arg_types = Vec::new();
                for arg in &args {
                    arg_types.push(arg.ty.clone());
                }
                
                let inference_arg_types: Vec<Type> = arg_types.iter().map(|a| a.clone().unwrap()).collect();
                
                gamma.add(bind.name.clone(), Type::Function(inference_arg_types.clone(), Box::new(Type::Int))); // Placeholder
                
                // args type map
                for (arg, arg_ty) in args.iter().zip(inference_arg_types.iter()) {
                    gamma.add(arg.name.clone(), arg_ty.clone());
                }
                
                let (body, body_ty) = body.to_typed_impl(gamma)?;
                
                // pop args type map
                for _ in &args {
                    gamma.pop();
                }
                
                let (body2, body2_ty) = body2.to_typed_impl(gamma)?;

                gamma.pop();
                
                Ok((typed_ast::Expr::LetFun {
                    bind: TypedIdent {
                        name: bind.name,
                        ty: Type::Function(inference_arg_types.clone(), Box::from(bind.ty.unwrap())),
                    },
                    args: args.into_iter().zip(inference_arg_types)
                        .map(|(arg, arg_ty)| TypedIdent {
                            name: arg.name,
                            ty: arg_ty.clone(),
                        }).collect(),
                    body: Box::new(body),
                    body2: Box::new(body2),
                }, body2_ty))
            }
            ASTExpr::Call { closure, args } => {
                let mut typed_args: Vec<typed_ast::Expr> = Vec::new();
                for arg in args {
                    let (current, current_ty) = arg.to_typed_impl(gamma)?;
                    typed_args.push(current);
                }
                
                let (closure, closure_ty) = closure.to_typed_impl(gamma)?;
                
                // TODO: type checking for function call
                let ret_ty: Type;
                
                match closure_ty {
                    Type::Function(expected_arg_types, expected_ret_ty) => {
                        ret_ty = *expected_ret_ty;
                    }
                    _ => {
                        return Err(FrontendError::TypeError("Expected a function".to_string()));
                    }
                }
                
                Ok((typed_ast::Expr::Call {
                    closure: Box::from(closure),
                    args: typed_args,
                    ret_ty: ret_ty.clone()
                }, ret_ty))
            }
        }
    }
}