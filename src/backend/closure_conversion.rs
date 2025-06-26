use std::rc::Rc;
use crate::backend::closure::{Closure, ClosureCExpr, ClosureExpr};
use crate::core::anf;
use crate::core::anf::CExpr;
use crate::util::name_generator::NameGenerator;

#[derive(Debug)]
pub enum ClosureProgramGlobal {
    FuncDef {
        closure: Rc<Closure>,
        body: ClosureExpr,
    }
}

#[derive(Debug)]
pub struct ClosureProgram {
    pub globals: Vec<ClosureProgramGlobal>,
    pub main: Option<ClosureExpr>,
    name_generator: NameGenerator
}

impl ClosureProgram {
    pub fn new() -> Self {
        ClosureProgram {
            globals: Vec::new(),
            main: None,
            name_generator: NameGenerator::new("lambda_")
        }
    }
    
    pub fn show(&self) -> String {
        let mut result = String::new();
        
        for global in &self.globals {
            match global {
                ClosureProgramGlobal::FuncDef { closure, body } => {
                    result.push_str(&format!("Function: {}\n", closure.global_name));
                    result.push_str(&format!("Args: {:?}\n", closure.args));
                    result.push_str(&format!("Capture: {:?}\n", closure.capture));
                    result.push_str(&format!("Return Type: {:?}\n", closure.ret_ty));
                    result.push_str(&format!("Body: {:?}\n\n", body));
                }
            }
        }
        
        if let Some(main) = &self.main {
            result.push_str(&format!("Main Function Body: {:?}\n", main));
        }
        
        result
    }
    
    pub fn convert(&mut self, anf_expr: anf::Expr) -> ClosureExpr {
        let converted_expr = self.convert_expr(anf_expr);
        
        if self.main.is_none() {
            self.main = Some(converted_expr.clone());
        }
        
        converted_expr
    }
    
    fn convert_cexpr(&mut self, cexpr: anf::CExpr) -> ClosureCExpr {
        match cexpr {
            anf::CExpr::Atom(atom) => ClosureCExpr::Atom(atom),
            anf::CExpr::Apply { func, args, ty } => {
                ClosureCExpr::Apply {
                    func,
                    args,
                    ty,
                }
            },
            anf::CExpr::If { cond, then, else_, ty } => ClosureCExpr::If {
                cond,
                then: Box::new(self.convert_expr(*then)),
                else_: Box::new(self.convert_expr(*else_)),
                ty,
            },
            anf::CExpr::Lambda { args, body, ret_ty } => {
                let free_vars = body.free_vars().into_iter().filter(|v| !args.contains(v)).collect::<Vec<_>>();
                
                let closure = Rc::new(Closure {
                    global_name: self.name_generator.next_name(),
                    args: args.clone(),
                    capture: free_vars,
                    ret_ty,
                });
                
                let body = self.convert_expr(*body);
                
                self.globals.push(ClosureProgramGlobal::FuncDef {
                    closure: closure.clone(),
                    body
                });
                
                ClosureCExpr::Closure(closure)
            }
        }
    }
    
    fn convert_expr(&mut self, expr: anf::Expr) -> ClosureExpr {
        match expr {
            anf::Expr::CExpr(cexpr) => ClosureExpr::CExpr(self.convert_cexpr(cexpr)),
            anf::Expr::Let { bind, value, body, ty, .. } => ClosureExpr::Let {
                bind,
                value: Box::new(self.convert_cexpr(*value)),
                body: Box::new(self.convert_expr(*body)),
                ty,
            },
        }
    }
}
