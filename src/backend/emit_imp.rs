use crate::backend::closure_conversion::ClosureBuilder;
use crate::backend::Error;
use crate::backend::Error::NonClosureError;
use crate::backend::imp::{ImpType, ImpVar};
use crate::backend::imp_builder::{FunctionHandle, ImpBuilder};
use crate::core::{anf, common};
use crate::core::anf::{CExpr};
use crate::core::common::OpType;
use crate::core::ty::Type;

pub fn emit_imp(program: ClosureBuilder) -> String {
    let mut builder = ImpBuilder::new();
    
    // Emit the functions
    for func in program.functions {
        let mut args_text_form = vec!["void* __env".to_string()];
        args_text_form.extend(func.clos.args.iter().map(|arg| format!("{} {}", arg.ty, arg.name)).collect::<Vec<_>>());
        
        // Signature
        builder.emit(
            format!("{} {}({}) {{", 
                ImpType::from_type(&func.clos.ret_ty),
                func.clos.global_name, 
                args_text_form.join(", ")
            )
        );
        
        builder.register_function(&*func.clos.global_name.to_string(), FunctionHandle {
            name: func.clos.global_name.clone(),
            args: func.clos.args.iter().map(|arg| { 
                ImpVar::from_typed_ident(arg)
            }).collect(),
            ret_ty: ImpType::from_type(&func.clos.ret_ty),
            captures: func.clos.capture.iter().map(|arg| {
                ImpVar::from_typed_ident(arg)
            }).collect(),
        });
        
        builder.push_scope();
        
        // Param binding
        for arg in &func.clos.args {
            let imp_var = ImpVar::from_typed_ident(arg);
            builder.initialize_var(&*arg.name, imp_var.clone());
            builder.push_var(&*arg.name, imp_var.clone());
            builder.emit(format!("{} = {};", imp_var.mangle(), arg.name));
        }
        
        // Capture binding
        let typed_env = builder.fresh_imp_var(ImpType::ClosureContextOf(func.clos.global_name.clone()));
        builder.emit(format!("{}* {} = ({}*) __env;", typed_env.ty, typed_env.mangle(), typed_env.ty));
        
        for capture in &func.clos.capture {
            let imp_var = ImpVar::from_typed_ident(capture);
            builder.initialize_var(&*capture.name, imp_var.clone());
            builder.push_var(&*capture.name, imp_var.clone());
            // load from closure context
            builder.emit(format!("{} = {}->{};", imp_var.mangle(), typed_env.mangle(), capture.name));
        }
        
        // Emit the body
        let body = func.body.emit_imp(&mut builder).unwrap().unwrap();
        
        // Emit a return statement
        builder.emit(format!("return {};", body.mangle()));
        
        // Close the function
        builder.pop_scope();
        
        // pop the param binding
        for _ in 0..(func.clos.args.len() + func.clos.capture.len()) {
            builder.pop_var();
        }
        
        // pop myself
        builder.pop_var();

        builder.emit("}\n".to_string());
    }
    
    // Emit the main function
    if let Some(main) = program.main {
        builder.emit("int main() {".to_string());
        builder.push_scope();
        
        // Emit the main body
        let main_body = main.emit_imp(&mut builder).unwrap().unwrap();
        
        // A temporary solution: print the result
        builder.emit(format!("printf(\"%d\\n\", {});", main_body.mangle()));
        
        // Close the main function
        builder.pop_scope();
        builder.emit("}\n".to_string());
    }
    
    builder.into_code()
}

pub trait EmitImp {
    type Output;
    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, Error>;
}

impl EmitImp for anf::Expr {
    type Output = Option<ImpVar>;

    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, Error> {
        match self {
            anf::Expr::CExpr(cexp) => {
                cexp.emit_imp(builder).map(|out| Some(out))
            }
            anf::Expr::Let { bind, value, body } => {
                // Evaluate the value
                let value_var = value.emit_imp(builder)?;
                
                // Bind the value to a new variable
                builder.push_var(&*bind.name, value_var);
                
                // Emit the body
                let result = body.emit_imp(builder);
                
                // Pop the variable binding
                builder.pop_var();
                
                result
            }
        }
    }
}

impl EmitImp for anf::CExpr {
    type Output = ImpVar;

    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, Error> {
        match self {
            CExpr::Atom(atom) => {
                atom.emit_imp(builder)
            }
            CExpr::Op { op, args } => {
                let mut args_imp_var = Vec::new();
                
                for arg in args {
                    let arg_imp_var = arg.emit_imp(builder)?;
                    args_imp_var.push(arg_imp_var);
                }

                let result = builder.fresh_imp_var(ImpType::Int);
                builder.emit(format!("{} {};", result.ty, result.mangle()));
                
                match op {
                    OpType::Add => {
                        builder.emit(format!("{} = {} + {};", result.mangle(), args_imp_var[0].mangle(), args_imp_var[1].mangle()));
                    }
                    OpType::Eq => {
                        builder.emit(format!("{} = {} == {};", result.mangle(), args_imp_var[0].mangle(), args_imp_var[1].mangle()));
                    }
                    OpType::Sub => {
                        builder.emit(format!("{} = {} - {};", result.mangle(), args_imp_var[0].mangle(), args_imp_var[1].mangle()));
                    }
                    OpType::Mul => {
                        builder.emit(format!("{} = {} * {};", result.mangle(), args_imp_var[0].mangle(), args_imp_var[1].mangle()));
                    }
                    OpType::Div => {
                        builder.emit(format!("{} = {} / {};", result.mangle(), args_imp_var[0].mangle(), args_imp_var[1].mangle()));
                    }
                }
                
                Ok(result)
            }
            CExpr::Call { closure, args, ret_ty } => {
                let clos_var = closure.emit_imp(builder)?;
                
                let signature_fn = |name: &str| format!(
                    "{} (*{})(void*, {})",
                    ImpType::from_type(&ret_ty),
                    name,
                    args.iter().map(|arg| format!("{}", ImpType::from_type(&arg.ty()))).collect::<Vec<_>>().join(", ")
                );
                
                // build the call args
                let mut call_args = Vec::new();
                for arg in args {
                    let arg_var = arg.emit_imp(builder)?;
                    call_args.push(arg_var);
                }
                
                let result = builder.fresh_imp_var(ImpType::from_type(&ret_ty));
                builder.emit(format!("{} {};", result.ty, result.mangle()));
                
                builder.emit(format!("{} = (({}) {}->func)({}->env, {});", 
                    result.mangle(),
                    signature_fn(""),
                    clos_var.mangle(),
                    clos_var.mangle(),
                    call_args.iter().map(|arg| arg.mangle()).collect::<Vec<_>>().join(", ")
                ));

                Ok(result)
            }
            CExpr::If { cond, then, else_, ty } => {
                let phi_var = builder.fresh_imp_var(ImpType::from_type(ty));
                builder.initialize_var(&*phi_var.name, phi_var.clone());
                
                let cond_var = cond.emit_imp(builder)?;
                
                builder.emit(format!("if ({}) {{", cond_var.mangle()));
                builder.push_scope();
                
                let then_var = then.emit_imp(builder)?.unwrap();
                builder.emit(format!("{} = {};", phi_var.mangle(), then_var.mangle()));
                
                builder.pop_scope();
                builder.emit(format!("}} else {{"));
                builder.push_scope();
                
                let else_var = else_.emit_imp(builder)?.unwrap();
                builder.emit(format!("{} = {};", phi_var.mangle(), else_var.mangle()));
                
                builder.pop_scope();
                builder.emit("}".to_string());
                
                Ok(phi_var)
            }
            CExpr::LetFun { .. } => {
                Err(NonClosureError("LetFun should not be emitted directly".to_string()))
            }
            CExpr::LetClos { bind, body } => {
                if let Type::Closure(clos) = &bind.ty {
                    let imp_clos_name = clos.global_name.clone();

                    let clos_var = ImpVar {
                        name: bind.name.clone(),
                        ty: ImpType::ClosureStruct,
                    };

                    let clos_env_var = builder.fresh_imp_var(ImpType::ClosureContextOf(imp_clos_name.clone()));

                    builder.emit(format!("{}* {} = malloc(sizeof({}));", clos_env_var.ty, clos_env_var.mangle(), clos_env_var.ty));

                    // Add the closure itself first,
                    // so that the closure can capture itself by name
                    builder.initialize_var(&*bind.name, clos_var.clone());
                    builder.push_var(
                        &bind.name.clone(),
                        clos_var.clone()
                    );

                    // fill the closure context
                    for capture in &clos.capture {
                        let _imp_arg = ImpVar::from_typed_ident(capture);
                        builder.initialize_var(&*capture.name, _imp_arg.clone());
                        builder.emit(format!("{}->{} = {};", clos_env_var.mangle(), capture.name, builder.resolve_var(&*capture.name)?.mangle()));
                    }

                    // Emit the closure context
                    builder.emit(format!("{}->func = (void *) {};", clos_var.mangle(), imp_clos_name));
                    builder.emit(format!("{}->env = (void *) {};", clos_var.mangle(), clos_env_var.mangle()));

                    // Emit the body
                    let result = body.emit_imp(builder);

                    // Pop the variable binding
                    builder.pop_var();

                    result.map(|opt| opt.unwrap())
                } else {
                    Err(NonClosureError("LetClos should be a closure".to_string()))
                }
            }
        }
    }
}

impl EmitImp for common::Atom {
    type Output = ImpVar;

    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, Error> {
        match self {
            common::Atom::Int(num) => {
                let imp_var = builder.fresh_imp_var(ImpType::Int);
                builder.initialize_var(&*imp_var.name, imp_var.clone());
                builder.emit(format!("{} = {};", imp_var.mangle(), num));
                Ok(imp_var)
            }
            common::Atom::Var(var) => {
                let imp_var = builder.resolve_var(&var.name)?;
                Ok(imp_var)
            }
            common::Atom::InputInt => {
                let imp_var = builder.fresh_imp_var(ImpType::Int);
                builder.initialize_var(&*imp_var.name, imp_var.clone());
                builder.emit(format!("scanf(\"%d\", &{});", imp_var.mangle()));
                Ok(imp_var)
            }
        }
    }
}