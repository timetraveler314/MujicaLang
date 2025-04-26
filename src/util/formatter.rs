use std::fmt;
use crate::backend::closure_conversion::{ClosureBuilder, GlobalFuncDef};
use crate::core::{anf, common};
use crate::core::anf::{CExpr, Expr};
use crate::core::ty::{Type, TypedIdent};

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

impl fmt::Display for common::Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            common::Atom::Int(i) => write!(f, "{}", i),
            common::Atom::Var(var) => write!(f, "{}", var.name),
            common::Atom::InputInt => write!(f, "input"),
        }
    }
}

impl fmt::Display for common::OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            common::OpType::Add => write!(f, "+"),
            common::OpType::Eq => write!(f, "=="),
            common::OpType::Sub => write!(f, "-"),
            common::OpType::Mul => write!(f, "*"),
        }
    }
}

impl anf::CExpr {
    fn ty(&self) -> Type {
        // Implementation of type inference would go here
        // This is just a placeholder for the formatter
        match self {
            CExpr::Atom(common::Atom::Int(_)) => Type::Int,
            CExpr::Atom(common::Atom::Var(v)) => v.ty.clone(),
            CExpr::Atom(common::Atom::InputInt) => Type::Int,
            CExpr::Op { op: common::OpType::Eq, .. } => Type::Int, // Assuming comparison returns int
            CExpr::Op { .. } => Type::Int,
            CExpr::Call { .. } => Type::Int, // Placeholder
            CExpr::If { then, .. } => then.ty(),
            CExpr::LetFun { bind, .. } => bind.ty.clone(),
            CExpr::LetClos { bind, .. } => bind.ty.clone(),
        }
    }

    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            CExpr::Atom(atom) => write!(f, "{:indent$}{}", "", atom, indent = indent),
            CExpr::Op { op, args } => {
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
            CExpr::Call { closure, args, ret_ty: _ret_ty } => {
                let args_str = args.iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{:indent$}{}({})", "", closure, args_str, indent = indent)
            }
            CExpr::If { cond, then, else_, .. } => {
                writeln!(f, "{:indent$}if {} then", "", cond, indent = indent)?;
                then.fmt_indented(f, indent + 2)?;
                writeln!(f, "\n{:indent$}else", "", indent = indent)?;
                else_.fmt_indented(f, indent + 2)
            }
            CExpr::LetFun { bind, args, body, body2 } => {
                write!(f, "{:indent$}let fun {}: {} ", "", bind.name, bind.ty, indent = indent)?;
                write!(f, "{}", args.iter()
                    .map(|a| format!("{}: {}", a.name, a.ty))
                    .collect::<Vec<_>>()
                    .join(" "))?;

                match **body {
                    Expr::CExpr(CExpr::Atom(_)) => {
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
            CExpr::LetClos { bind, body } => {
                write!(f, "{:indent$}let clos {}: {}", "", bind.name, bind.ty, indent = indent)?;
                writeln!(f, "\n{:indent$}in", "", indent = indent)?;
                body.fmt_indented(f, indent + 2)
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
    fn ty(&self) -> Type {
        // Implementation of type inference would go here
        // This is just a placeholder for the formatter
        match self {
            Expr::CExpr(cexpr) => cexpr.ty(),
            Expr::Let { bind, .. } => bind.ty.clone(),
        }
    }
    
    fn fmt_indented(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            Expr::CExpr(cexpr) => cexpr.fmt_indented(f, indent),

            Expr::Let { bind, value, body } => {
                write!(f, "{:indent$}let {}: {} = ", "", bind.name, bind.ty, indent = indent)?;
                if let CExpr::Atom(_) | CExpr::Op { .. } | CExpr::Call { .. } = **value {
                    value.fmt_indented(f, 0)?;
                } else {
                    writeln!(f)?;
                    value.fmt_indented(f, indent + 2)?;
                }
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
        // Print function header with type annotation
        write!(f, "fun {}: {} {{", self.clos.global_name, self.body.ty())?;

        // Print capture variables with types
        let captures: Vec<String> = self.clos.capture.iter()
            .map(|c| format!("{}: {}", c.name, c.ty))
            .collect();
        write!(f, "{}", captures.join(", "))?;

        write!(f, "}} ")?;

        // Print arguments with types
        let args: Vec<String> = self.clos.args.iter()
            .map(|a| format!("{}: {}", a.name, a.ty))
            .collect();
        write!(f, "{}", args.join(" "))?;

        // Handle body formatting
        match &*self.body {
            Expr::CExpr(CExpr::Atom(_)) => {
                write!(f, " = ")?;
                self.body.fmt(f)
            }
            _ => {
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
            writeln!(f, "\nmain: {} =", main_expr.ty())?;
            main_expr.fmt_indented(f, 2)?;
        }

        Ok(())
    }
}

