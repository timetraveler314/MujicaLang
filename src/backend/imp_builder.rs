use std::collections::{HashMap};
use crate::backend::{BackendError};
use crate::backend::BackendError::ImpError;
use crate::backend::imp::{ImpType, ImpVar, CLOSURE_NAME};
use crate::frontend::name_resolution::NameIdentifier;

#[derive(Debug)]
#[derive(Eq, Hash, PartialEq)]
pub struct FunctionHandle {
    pub name: String,
    pub args: Vec<ImpVar>,
    pub ret_ty: ImpType,
    pub captures: Vec<ImpVar>,
}

#[derive(Debug)]
pub enum ImpLine {
    String(String),
}

#[derive(Debug, Default)]
pub struct ImpBuilder {
    // Tracks variable assignments
    variables: HashMap<NameIdentifier, ImpVar>,
    // Generated C statements
    statements: Vec<ImpLine>,
    // Counter for generating fresh variable names
    temp_counter: usize,
    // Set of global function declarations
    functions: HashMap<String, FunctionHandle>,
    // Indentation level for pretty printing
    indent_level: usize,
    // Maps closure names to their C struct definitions
    closures: HashMap<String, (String, Vec<ImpVar>)>,
    // Maps variables to their closure contexts
    context_map: HashMap<String, String>,
}

impl ImpBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Generate a fresh temporary variable name
    pub fn fresh_imp_var(&mut self, ty: ImpType) -> ImpVar {
        let name = format!("tmp{}", self.temp_counter);
        self.temp_counter += 1;

        // The ImpVar is a C variable
        let imp_var = ImpVar {
            name: name.clone(),
            ty
        };

        imp_var
    }

    fn push_line(&mut self, line: String) {
        self.statements.push(ImpLine::String(line));
    }

    /// Add a C statement to the output
    pub fn emit(&mut self, stmt: String) {
        self.push_line(format!("{}{}", " ".repeat(self.indent_level * 4), stmt));
    }

    /// Get the C variable name for an ANF variable
    pub fn resolve_var(&self, var: &NameIdentifier) -> Result<ImpVar, BackendError> {
        if let Some(imp_var) = self.variables.get(var) {
            Ok(imp_var.clone())
        } else {
            Err(ImpError(format!("Variable {} not found in current scope", var.0)))
        }
    }

    // pub fn initialize_var(&mut self, anf_var: &NameIdentifier, imp_var: ImpVar) {
    //     if !self.variables.contains_key(anf_var) {
    //         let should_initialize = match imp_var.ty {
    //             ImpType::ClosureContextOf(_) => true,
    //             ImpType::ClosureStruct => true,
    //             _ => false,
    //         };
    //         if should_initialize {
    //             self.emit(format!("{}* {} = malloc(sizeof({}));", imp_var.ty, imp_var.name, imp_var.ty));
    //         } else {
    //             self.emit(format!("{} {}; // {}", imp_var.ty, imp_var.name, anf_var.0));
    //         }
    //     }
    // }
    pub fn initialize_var(&mut self, imp_var: ImpVar) {
        let should_initialize = match imp_var.ty {
            ImpType::ClosureContextOf(_) => true,
            ImpType::ClosureStruct => true,
            _ => false,
        };
        
        if should_initialize {
            self.emit(format!("{}* {} = malloc(sizeof({}));", imp_var.ty, imp_var.name, imp_var.ty));
        } else {
            self.emit(format!("{} {};", imp_var.ty, imp_var.name));
        }
    }

    /// Bind a new ANF variable to a C variable
    pub fn push_var(&mut self, anf_var: &NameIdentifier, imp_var: ImpVar) {
        self.variables.insert(anf_var.clone(), imp_var.clone());
    }

    pub fn push_scope(&mut self) {
        self.indent_level += 1;
    }

    pub fn pop_scope(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn register_function(&mut self, name: &str, handle: FunctionHandle) {
        self.functions.insert(name.to_string(), handle);
    }

    pub fn register_closure(&mut self, name: &str, clos_name: &str, args: Vec<ImpVar>) {
        self.closures.insert(name.to_string(), (clos_name.to_string(), args));
        self.context_map.insert(name.to_string(), clos_name.to_string());
    }

    pub fn get_closure_context(&self, name: &str) -> Option<String> {
        self.context_map.get(name).cloned()
    }

    /// Get all generated imperative code
    pub fn into_code(self) -> String {
        let mut code = String::new();

        // C Libraries <stdio> and <stdlib.h>
        code.push_str("#include <stdio.h>\n");
        code.push_str("#include <stdlib.h>\n");

        // Globally, a closure is a function pointer (void *)
        // and a pointer to the closure context (also void *)
        // Emit closure struct definitions
        code.push_str(format!(
            r#"
typedef struct {{
    void* func;
    void* env;
}} {};
"#,
            CLOSURE_NAME
        ).as_str());

        // Emit closure struct definitions
        for (name, handle) in self.functions.iter() {
            let captures = &handle.captures;

            let capture_list = captures.iter()
                .map(|arg| format!("{} {}", arg.ty, arg.name))
                .collect::<Vec<_>>()
                .join(";\n    ");

            code.push_str(format!(
                r#"
typedef struct {{
    {};
}} clos_env_{};

"#,
                capture_list,
                name
            ).as_str());
        }

        // Emit function declarations first
        for (_, func_handle) in &self.functions {
            let decl = format!(
                "{} {}(void* __env, {});",
                func_handle.ret_ty,
                func_handle.name,
                func_handle.args.iter()
                    .map(|arg| format!("{} {}", arg.ty, arg.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            );

            code.push_str(&format!("{}\n", decl));
        }

        // Then emit the main code
        for line in &self.statements {
            match line {
                ImpLine::String(line) => {
                    code.push_str(&line);
                    code.push('\n');
                }
            }
        }
        // code.push_str(&self.statements.join("\n"));
        code
    }
}