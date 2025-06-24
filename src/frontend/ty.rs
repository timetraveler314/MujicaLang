use std::fmt;
use std::fmt::Display;

pub type TypeVar = String;

#[derive(Debug, Clone)]
pub enum Ty {
    Unit,
    Int,
    Bool,
    Arrow(Box<Ty>, Box<Ty>),

    /// A monomorphic type, or a type variable
    Mono(TypeVar),
}

impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Unit => write!(f, "unit"),
            Ty::Int => write!(f, "int"),
            Ty::Bool => write!(f, "bool"),
            Ty::Mono(tv) => write!(f, "{tv}"),
            Ty::Arrow(t1, t2) => {
                // Add parentheses around the left type if it is another Arrow
                let left = match **t1 {
                    Ty::Arrow(_, _) => format!("({})", t1),
                    _ => format!("{}", t1),
                };
                write!(f, "{} -> {}", left, t2)
            }
        }
    }
}

impl Ty {
    pub fn contains_var(&self, var: &TypeVar) -> bool {
        match self {
            Ty::Unit | Ty::Int | Ty::Bool => false,
            Ty::Arrow(left, right) => left.contains_var(var) || right.contains_var(var),
            Ty::Mono(v) => v == var,
        }
    }
    
    pub fn apply(&self, var: &TypeVar, ty: &Ty) -> Ty {
        match self {
            Ty::Mono(v) if v == var => ty.clone(),
            Ty::Arrow(left, right) => Ty::Arrow(
                Box::new(left.apply(var, ty)),
                Box::new(right.apply(var, ty)),
            ),
            _ => self.clone(),
        }
    }
}

/// Represents a type scheme, which is a type with a set of universally quantified type variables
#[derive(Debug)]
pub struct Scheme {
    pub vars: Vec<TypeVar>,
    pub ty: Ty,
}

#[derive(Debug)]
pub struct TypingContext<T> {
    mapping: Vec<(String, T)>,
}

impl<T> TypingContext<T> {
    pub fn new() -> Self {
        TypingContext { mapping: Vec::new() }
    }

    pub fn push(&mut self, name: &str, value: T) {
        self.mapping.push((name.to_string(), value));
    }

    pub fn lookup(&self, name: &str) -> Option<&T> {
        self.mapping.iter().find_map(|(n, v)| if n == name { Some(v) } else { None })
    }

    pub fn pop(&mut self) {
        self.mapping.pop();
    }
}

pub type MonoContext = TypingContext<Ty>;
pub type SchemeContext = TypingContext<Scheme>;