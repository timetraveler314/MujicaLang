use std::process::id;
use crate::backend::BackendError;
use crate::backend::closure::{ClosureCExpr, ClosureExpr};
use crate::backend::closure::Closure;
use crate::backend::closure_conversion::{ClosureProgram, ClosureProgramGlobal};
use crate::backend::imp::{ImpType, ImpVar};
use crate::backend::imp_builder::{FunctionHandle, ImpBuilder};
use crate::core::{Atom, TypedAtom};
use crate::frontend::ast::OpType;
use crate::frontend::ty::Ty;

pub fn emit_imp(program: ClosureProgram) -> String {
    let mut builder = ImpBuilder::new();

    // Emit the functions
    for global_entry in program.globals {
        match global_entry {
            ClosureProgramGlobal::FuncDef { closure, body } => {
                let mut args_text_form = vec!["void* __env".to_string()];
                args_text_form.extend(closure.args.iter().map(|arg| {
                    let imp_var = ImpVar::from_typed_ident(arg);
                    format!("{} {}", imp_var.ty, imp_var.name)
                }).collect::<Vec<_>>());

                // Signature
                builder.emit(
                    format!("{} {}({}) {{",
                            ImpType::from_type(&closure.ret_ty),
                            closure.global_name,
                            args_text_form.join(", ")
                    )
                );

                builder.register_function(&closure.global_name, FunctionHandle {
                    name: closure.global_name.clone(),
                    args: closure.args.iter().map(|arg| {
                        ImpVar::from_typed_ident(arg)
                    }).collect(),
                    ret_ty: ImpType::from_type(&closure.ret_ty),
                    captures: closure.capture.iter().map(|arg| {
                        ImpVar::from_typed_ident(arg)
                    }).collect(),
                });

                builder.push_scope();

                // Param binding
                for arg in &closure.args {
                    let imp_var = ImpVar::from_typed_ident(arg);
                    // builder.initialize_var(&*arg.name, imp_var.clone());
                    builder.push_var(&arg.0.id, imp_var.clone());
                    // builder.emit(format!("{} = {};", imp_var.mangle(), arg.name));
                }

                // Capture binding
                let typed_env = builder.fresh_imp_var(ImpType::ClosureContextOf(closure.global_name.clone()));
                builder.emit(format!("{}* {} = ({}*) __env;", typed_env.ty, typed_env.name, typed_env.ty));

                for capture in &closure.capture {
                    let imp_var = ImpVar::from_typed_ident(capture);
                    builder.initialize_var(imp_var.clone()); // &capture.0.id, 
                    builder.push_var(&capture.0.id, imp_var.clone());
                    // load from closure context
                    builder.emit(format!("{} = {}->{};", imp_var.name, typed_env.name, imp_var.name)); // To CHECK
                }

                // Emit the body
                let body = body.emit_imp(&mut builder).unwrap().unwrap();

                // Emit a return statement
                builder.emit(format!("return {};", body.name));

                // Close the function
                builder.pop_scope();

                builder.emit("}\n".to_string());
            }
        }
    }

    // Emit the main function
    if let Some(main) = program.main {
        builder.emit("int main() {".to_string());
        builder.push_scope();

        // Emit the main body
        let main_body = main.emit_imp(&mut builder).unwrap().unwrap();

        // A temporary solution: print the result
        builder.emit(format!("printf(\"%d\\n\", {});", main_body.name));

        // Close the main function
        builder.pop_scope();
        builder.emit("}\n".to_string());
    }

    builder.into_code()
}

pub trait EmitImp {
    type Output;
    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, BackendError>;
}

impl EmitImp for ClosureExpr {
    type Output = Option<ImpVar>;

    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, BackendError> {
        match self {
            ClosureExpr::CExpr(cexp) => {
                cexp.emit_imp(builder).map(|out| Some(out))
            }
            ClosureExpr::Let { bind, value, body, ty } => {
                let is_closure = match value.as_ref() {
                    ClosureCExpr::Closure(clos) => Some(clos),
                    _ => None,
                };
                
                if let Some(clos) = is_closure {
                    let imp_clos_name = clos.global_name.clone();

                    let clos_var = ImpVar {
                        name: format!("{}_{}_clos", bind.name, bind.id.0),
                        // ty: ImpType::from_type(&ty),
                        ty: ImpType::ClosureStruct,
                    };

                    let clos_env_var = builder.fresh_imp_var(ImpType::ClosureContextOf(imp_clos_name.clone()));

                    builder.emit(format!("{}* {} = malloc(sizeof({}));", clos_env_var.ty, clos_env_var.name, clos_env_var.ty));

                    // Add the closure itself first,
                    // so that the closure can capture itself by name
                    builder.initialize_var(clos_var.clone()); // &*bind.name, 
                    builder.push_var(
                        &bind.id,
                        clos_var.clone()
                    );

                    // fill the closure context
                    for capture in &clos.capture {
                        let _imp_arg = ImpVar::from_typed_ident(capture);
                        builder.initialize_var(_imp_arg.clone()); // &*capture.name, 
                        builder.emit(format!("{}->{} = {};", clos_env_var.name, _imp_arg.name, builder.resolve_var(&capture.0.id)?.name));
                    }

                    // Emit the closure context
                    builder.emit(format!("{}->func = (void *) {};", clos_var.name, imp_clos_name));
                    builder.emit(format!("{}->env = (void *) {};", clos_var.name, clos_env_var.name));

                    // Emit the body
                    let result = body.emit_imp(builder);

                    result
                } else {
                    // Evaluate the value
                    let value_var = value.emit_imp(builder)?;

                    // Bind the value to a new variable
                    builder.push_var(&bind.id, value_var);

                    // Emit the body
                    let result = body.emit_imp(builder);

                    return result;
                }
            }
        }
    }
}

impl EmitImp for ClosureCExpr {
    type Output = ImpVar;

    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, BackendError> {
        match self {
            ClosureCExpr::Atom(atom) => {
                atom.emit_imp(builder)
            }
            ClosureCExpr::Apply { func, args, ty } => {
                // build the call args
                let mut call_args = Vec::new();
                for arg in args {
                    let arg_var = arg.emit_imp(builder)?;
                    call_args.push(arg_var);
                }

                let result = builder.fresh_imp_var(ImpType::from_type(&ty));
                builder.emit(format!("{} {};", result.ty, result.name));
                
                // Emit according to the function type
                match &func.atom {
                    Atom::Var(var) => {
                        let clos_var = builder.resolve_var(&var.id)?;
                        
                        let signature_fn = |name: &str| format!(
                            "{} (*{})(void*, {})",
                            ImpType::from_type(&ty),
                            name,
                            args.iter().map(|arg| format!("{}", ImpType::from_type(&arg.ty))).collect::<Vec<_>>().join(", ")
                        );

                        builder.emit(format!("{} = (({}) {}->func)({}->env, {});",
                                             result.name,
                                             signature_fn(""),
                                             clos_var.name,
                                             clos_var.name,
                                             call_args.iter().map(|arg| arg.name.clone()).collect::<Vec<_>>().join(", ")
                        ));
                    }
                    Atom::Op(op) => {
                        // Logic for applying an operator

                        macro_rules! emit_binary_op {
                            ($builder:expr, $result:expr, $args:expr, $op:tt) => {
                                $builder.emit(format!("{} = {} {} {};", 
                                    $result.name, 
                                    $args[0].name, 
                                    $op, 
                                    $args[1].name
                                ))
                            };
                        }

                        match op {
                            OpType::Add => { emit_binary_op!(builder, result, call_args, "+"); }
                            OpType::Sub => { emit_binary_op!(builder, result, call_args, "-"); }
                            OpType::Mul => { emit_binary_op!(builder, result, call_args, "*"); }
                            OpType::Div => { emit_binary_op!(builder, result, call_args, "/"); }
                            OpType::Eq => { emit_binary_op!(builder, result, call_args, "=="); }
                            _ => unimplemented!(),
                        }
                    }
                    Atom::Unit | Atom::Int(_) | Atom::Bool(_) => {
                        return Err(BackendError::ImpError("Cannot apply non-function".to_string()));
                    }
                }

                Ok(result)
            }
            ClosureCExpr::If { cond, then, else_, ty } => {
                let phi_var = builder.fresh_imp_var(ImpType::from_type(ty));
                builder.initialize_var(phi_var.clone()); // &*phi_var.name,

                let cond_var = (TypedAtom { atom: cond.clone(), ty: Ty::Bool }).emit_imp(builder)?;

                builder.emit(format!("if ({}) {{", cond_var.name));
                builder.push_scope();

                let then_var = then.emit_imp(builder)?.unwrap();
                builder.emit(format!("{} = {};", phi_var.name, then_var.name));

                builder.pop_scope();
                builder.emit(format!("}} else {{"));
                builder.push_scope();

                let else_var = else_.emit_imp(builder)?.unwrap();
                builder.emit(format!("{} = {};", phi_var.name, else_var.name));

                builder.pop_scope();
                builder.emit("}".to_string());

                Ok(phi_var)
            }
            ClosureCExpr::Closure(_) => unreachable!(),
        }
    }
}

impl EmitImp for TypedAtom {
    type Output = ImpVar;

    fn emit_imp(&self, builder: &mut ImpBuilder) -> Result<Self::Output, BackendError> {
        let atom = &self.atom;
        
        match atom {
            // common::Atom::Int(num) => {
            //     let imp_var = builder.fresh_imp_var(ImpType::Int);
            //     builder.initialize_var(&*imp_var.name, imp_var.clone());
            //     builder.emit(format!("{} = {};", imp_var.mangle(), num));
            //     Ok(imp_var)
            // }
            // common::Atom::Var(var) => {
            //     let imp_var = builder.resolve_var(&var.name)?;
            //     Ok(imp_var)
            // }
            // common::Atom::InputInt => {
            //     let imp_var = builder.fresh_imp_var(ImpType::Int);
            //     builder.initialize_var(&*imp_var.name, imp_var.clone());
            //     builder.emit(format!("scanf(\"%d\", &{});", imp_var.mangle()));
            //     Ok(imp_var)
            // }
            Atom::Int(num) => {
                let imp_var = builder.fresh_imp_var(ImpType::Int);
                builder.initialize_var(imp_var.clone());
                builder.emit(format!("{} = {};", imp_var.name, num));
                Ok(imp_var)
            }
            Atom::Var(var) => {
                let imp_var = builder.resolve_var(&var.id)?;
                Ok(imp_var)
            }
            Atom::Op(_) => unreachable!(),
            Atom::Unit => unimplemented!(),
            Atom::Bool(bool) => {
                let imp_var = builder.fresh_imp_var(ImpType::Int);
                builder.initialize_var(imp_var.clone());
                builder.emit(format!("{} = {};", imp_var.name, if *bool { 1 } else { 0 }));
                Ok(imp_var)
            }
        }
    }
}