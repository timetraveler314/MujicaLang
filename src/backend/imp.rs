use std::fmt::Display;
use crate::frontend::name_resolution::ResolvedIdent;
use crate::frontend::ty::Ty;

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

    pub fn from_type(ty: &Ty) -> Self {
        match ty {
            Ty::Int => ImpType::Int,
            Ty::Unit => ImpType::Void,
            Ty::Bool => ImpType::Int,
            Ty::Arrow(..) => {
                ImpType::Ptr(
                    Box::new(ImpType::ClosureStruct)
                )
            }
            Ty::Mono(..) => panic!("Cannot convert mono type to imp type"),
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
    pub fn from_typed_ident((ident, ty): &(ResolvedIdent, Ty)) -> Self {
        ImpVar {
            name: format!("{}_{}", ident.name, ident.id.0),
            ty: ImpType::from_type(&ty),
        }
    }
}