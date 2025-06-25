use std::collections::HashMap;
use crate::frontend::ast::{ASTAtom, ASTExpr, InputASTExpr};
use crate::frontend::FrontendError;
use crate::frontend::ty::Ty;

pub type ResolvedASTExpr = ASTExpr<ResolvedIdent, Option<Ty>>;

#[derive(Debug, Clone)]
pub struct ResolvedIdent {
    pub name: String,
    pub id: usize,
}

impl ResolvedIdent {
    pub fn new(name: String, id: usize) -> Self {
        ResolvedIdent { name, id }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl PartialEq<Self> for ResolvedIdent {
    fn eq(&self, other: &Self) -> bool {
        // Only judge equality based on the identifier's id
        self.id == other.id
    }
}

impl Eq for ResolvedIdent {}

pub struct NameResolver {
    counter: usize,
    scopes: Vec<HashMap<String, ResolvedIdent>>,
}

impl NameResolver {
    pub fn new() -> Self {
        // With an empty global scope
        NameResolver { counter: 0, scopes: vec![HashMap::new()] }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup_ident(&self, name: &str) -> Option<ResolvedIdent> {
        for scope in self.scopes.iter().rev() {
            if let Some(ident) = scope.get(name) {
                return Some(ident.clone());
            }
        }
        None
    }

    fn insert_ident(&mut self, name: String) -> ResolvedIdent {
        let id = self.counter;
        self.counter += 1;
        let ident = ResolvedIdent::new(name, id);

        self.scopes
            .last_mut()
            .expect("No scope to insert identifier into")
            .insert(ident.name.clone(), ident.clone());

        ident
    }

    pub fn resolve(&mut self, ast: InputASTExpr) -> Result<ResolvedASTExpr, FrontendError> {
        match ast {
            InputASTExpr::Atom(atom, ty) => {
                match atom {
                    ASTAtom::Var(name) => {
                        if let Some(ident) = self.lookup_ident(&name) {
                            Ok(ASTExpr::Atom(ASTAtom::Var(ident), ty))
                        } else {
                            // Unbound variable
                            Err(FrontendError::UnboundVariable(name))
                        }
                    },
                    ASTAtom::Int(int) => Ok(ASTExpr::Atom(ASTAtom::Int(int), ty)),
                    ASTAtom::Op(op) => Ok(ASTExpr::Atom(ASTAtom::Op(op), ty)),
                }
            }
            InputASTExpr::If { cond, then, else_, ty } => {
                Ok(ASTExpr::If {
                    cond: Box::new(self.resolve(*cond)?),
                    then: Box::new(self.resolve(*then)?),
                    else_: Box::new(self.resolve(*else_)?),
                    ty,
                })
            }
            InputASTExpr::Let { bind: (ident, bind_ty), value, body, ty } => {
                // First resolve value without inserting the binding
                let resolved_value = self.resolve(*value)?;
                
                // Push a new scope for the let binding
                self.push_scope();
                
                // Insert the binding into the current scope
                let resolved_ident = self.insert_ident(ident.clone());
                
                // Resolve the body with the new binding
                let resolved_body = self.resolve(*body)?;
                
                // Pop the scope after resolving the body
                self.pop_scope();
                
                // Return the resolved let expression
                Ok(ASTExpr::Let {
                    bind: (resolved_ident, bind_ty),
                    value: Box::new(resolved_value),
                    body: Box::new(resolved_body),
                    ty,
                })
            }
            InputASTExpr::Apply { func, args, ty } => {
                let resolved_func = self.resolve(*func)?;
                let resolved_args = self.resolve(*args)?;

                Ok(ASTExpr::Apply {
                    func: Box::new(resolved_func),
                    args: Box::new(resolved_args),
                    ty,
                })
            }
            InputASTExpr::Lambda { arg, body, ret_ty } => {
                // Push a new scope for the lambda
                self.push_scope();
                
                // Insert the argument into the current scope
                let resolved_ident = self.insert_ident(arg.0.clone());
                
                // Resolve the body with the new binding
                let resolved_body = self.resolve(*body)?;
                
                // Pop the scope after resolving the body
                self.pop_scope();
                
                // Return the resolved lambda expression
                Ok(ASTExpr::Lambda {
                    arg: (resolved_ident, arg.1),
                    body: Box::new(resolved_body),
                    ret_ty,
                })
            }
        }
    }
}