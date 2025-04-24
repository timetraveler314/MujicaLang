use std::fmt;
use crate::backend::closure_conversion::{ClosureBuilder, GlobalFuncDef};
use crate::core::anf;
use crate::core::ty::TypedIdent;

impl fmt::Display for crate::core::ty::Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            crate::core::ty::Type::Int => write!(f, "int"),
            crate::core::ty::Type::Unit => write!(f, "unit"),
            crate::core::ty::Type::Function(params, ret) => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            crate::core::ty::Type::Closure(closure) => {
                write!(f, "closure[{}]", closure.global_name)
            }
        }
    }
}

impl fmt::Display for TypedIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl fmt::Display for anf::Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            anf::Atom::Int(i) => write!(f, "{}", i),
            anf::Atom::Var(var) => write!(f, "{}", var.name),
        }
    }
}

impl fmt::Display for anf::OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            anf::OpType::Add => write!(f, "+"),
            anf::OpType::Eq => write!(f, "=="),
            anf::OpType::Sub => write!(f, "-"),
            anf::OpType::Mul => write!(f, "*"),
        }
    }
}

impl anf::CExpr {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            anf::CExpr::Atom(atom) => write!(f, "{:indent$}{}", "", atom, indent = indent),
            anf::CExpr::Op { op, args } => {
                if args.len() == 2 {
                    write!(f, "{:indent$}{} {} {}", "", args[0], op, args[1], indent = indent)
                } else {
                    let args_str = args.iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "{:indent$}{}({})", "", op, args_str, indent = indent)
                }
            }
            anf::CExpr::Call { closure, args } => {
                let args_str = args.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{:indent$}{}({})", "", closure, args_str, indent = indent)
            }
            anf::CExpr::If { cond, then, else_ } => {
                writeln!(f, "{:indent$}if {} then", "", cond, indent = indent)?;
                then.fmt_indented(f, indent + 2)?;
                writeln!(f, "\n{:indent$}else", "", indent = indent)?;
                else_.fmt_indented(f, indent + 2)
            }
        }
    }
}

impl fmt::Display for anf::CExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl anf::Expr {
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            anf::Expr::CExpr(cexpr) => cexpr.fmt_indented(f, indent),

            anf::Expr::Let { bind, value, body } => {
                write!(f, "{:indent$}let {} = ", "", bind.name, indent = indent)?;
                if let anf::CExpr::Atom(_) | anf::CExpr::Op { .. } | anf::CExpr::Call { .. } = **value {
                    value.fmt_indented(f, 0)?;
                } else {
                    writeln!(f)?;
                    value.fmt_indented(f, indent + 2)?;
                }
                writeln!(f, "\n{:indent$}in", "", indent = indent)?;
                body.fmt_indented(f, indent + 2)
            }

            anf::Expr::LetFun { bind, args, body, body2 } => {
                write!(f, "{:indent$}let fun {} ", "", bind.name, indent = indent)?;
                write!(f, "{}", args.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(" "))?;

                match **body {
                    anf::Expr::CExpr(anf::CExpr::Atom(_)) => {
                        write!(f, " = ")?;
                        body.fmt_indented(f, 0)?;
                    }
                    _ => {
                        writeln!(f, " =")?;
                        body.fmt_indented(f, indent + 2)?;
                    }
                }

                writeln!(f, "\n{:indent$}in", "", indent = indent)?;
                body2.fmt_indented(f, indent + 2)
            }

            anf::Expr::LetClos { bind, body } => {
                write!(f, "{:indent$}let clos {}", "", bind.name, indent = indent)?;
                writeln!(f, "\n{:indent$}in", "", indent = indent)?;
                body.fmt_indented(f, indent + 2)
            }
        }
    }
}

impl fmt::Display for anf::Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_indented(f, 0)
    }
}

impl fmt::Display for anf::Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{", self.global_name)?;
        write!(f, "{}", self.capture.iter().map(|c| c.name.clone()).collect::<Vec<_>>().join(", "))?;
        write!(f, "}} ")?;
        write!(f, "{}", self.args.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(" "))
    }
}
impl fmt::Display for GlobalFuncDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print function header: fun name {captures} params =
        write!(f, "fun {} {{", self.clos.global_name)?;

        // Print capture variables
        let captures: Vec<String> = self.clos.capture.iter()
            .map(|c| c.name.clone())
            .collect();
        write!(f, "{}", captures.join(", "))?;

        write!(f, "}} ")?;

        // Print arguments
        let args: Vec<String> = self.clos.args.iter()
            .map(|a| a.name.clone())
            .collect();
        write!(f, "{}", args.join(" "))?;

        // Handle body formatting
        match &*self.body {
            anf::Expr::CExpr(anf::CExpr::Atom(_)) => {
                // Simple body on same line
                write!(f, " = ")?;
                self.body.fmt(f)
            }
            _ => {
                // Complex body on new line with indentation
                writeln!(f, " =")?;
                self.body.fmt_indented(f, 2)
            }
        }
    }
}

impl fmt::Display for ClosureBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print all global function definitions
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }

        // Print the main expression if it exists
        if let Some(main_expr) = &self.main {
            writeln!(f, "\nmain =")?;
            main_expr.fmt_indented(f, 2)?;
        }

        Ok(())
    }
}
