use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Display;
use crate::frontend::tyck::type_class::TypeClassConstraint;

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

    pub fn free_vars(&self) -> HashSet<TypeVar> {
        match self {
            Ty::Unit | Ty::Int | Ty::Bool => HashSet::new(),
            Ty::Mono(var) => {
                let mut vars = HashSet::new();
                vars.insert(var.clone());
                vars
            }
            Ty::Arrow(left, right) => {
                let mut vars = left.free_vars();
                vars.extend(right.free_vars());
                vars
            }
        }
    }
}

/// Represents a type scheme, which is a type with a set of universally quantified type variables
/// `forall t_1, ..., t_n, [C_1, ..., C_m] . T`
#[derive(Debug, Clone)]
pub struct Scheme {
    pub vars: Vec<TypeVar>,
    pub constraints: Vec<TypeClassConstraint>,
    pub ty: Ty,
}

impl Display for Scheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Print type variables
        let vars = if self.vars.is_empty() {
            "".to_string()
        } else {
            format!("forall {}. ", self.vars.join(" "))
        };

        // Print constraints
        let constraints = if self.constraints.is_empty() {
            "".to_string()
        } else {
            let cs: Vec<String> = self.constraints.iter().map(|c| c.to_string()).collect();
            format!("({}) => ", cs.join(", "))
        };

        write!(f, "{}{}{}", vars, constraints, self.ty)
    }
}


#[derive(Debug)]
pub struct TypingContext {
    mapping: HashMap<usize, Scheme>
}

impl TypingContext {
    pub fn new() -> Self {
        TypingContext {
            mapping: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: usize, scheme: Scheme) {
        self.mapping.insert(id, scheme);
    }

    pub fn get(&self, id: &usize) -> Option<&Scheme> {
        self.mapping.get(id)
    }

    pub fn contains(&self, id: &usize) -> bool {
        self.mapping.contains_key(id)
    }
}