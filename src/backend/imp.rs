use std::fmt::Display;
use crate::core::ty::{Type, TypedIdent};

pub const CLOSURE_NAME: &str = "__closure";

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum ImpType {
    Int,
    Void,
    Struct(String),
    Ptr(Box<ImpType>),
    ClosureContextOf(String),
    ClosureStruct,
}

impl ImpType {
    pub fn void_ptr() -> ImpType {
        ImpType::Ptr(Box::new(ImpType::Void))
    }
    
    pub fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Int => ImpType::Int,
            Type::Unit => ImpType::Void,
            Type::Function(..) => {
                unreachable!()
            }
            Type::Closure(closure) => {
                ImpType::ClosureContextOf(closure.global_name.clone())
            }
        }
    }
}

impl Display for ImpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ImpType::Int => "int".to_string(),
            ImpType::Void => "void".to_string(),
            ImpType::Struct(name) => format!("struct {}", name),
            ImpType::Ptr(ty) => format!("{}*", ty.to_string()),
            ImpType::ClosureContextOf(name) => format!("clos_env_{}", name),
            ImpType::ClosureStruct => CLOSURE_NAME.to_string(),
        };
        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ImpVar {
    pub name: String,
    pub ty: ImpType,
}

impl ImpVar {
    pub fn mangle(&self) -> String {
        format!("{}__{}", self.name, self.mangle_type())
    }

    fn mangle_type(&self) -> String {
        match &self.ty {
            ImpType::Int => "i".to_string(),
            ImpType::Void => "v".to_string(),
            ImpType::Struct(name) => format!("s{}", name),
            ImpType::Ptr(ty) => format!("p{}", ty.to_string()),
            ImpType::ClosureContextOf(name) => format!("C{}__", name),
            ImpType::ClosureStruct => CLOSURE_NAME.to_string(),
        }
    }
    
    pub fn from_typed_ident(ident: &TypedIdent) -> Self {
        ImpVar {
            name: ident.name.clone(),
            ty: ImpType::from_type(&ident.ty),
        }
    }
}