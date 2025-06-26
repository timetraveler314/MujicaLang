use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use crate::frontend::name_resolution::NameIdentifier;
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

impl PartialEq for Ty {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Ty::Unit, Ty::Unit) => true,
            (Ty::Int, Ty::Int) => true,
            (Ty::Bool, Ty::Bool) => true,
            (Ty::Arrow(l1, r1), Ty::Arrow(l2, r2)) => { l1 == l2 && r1 == r2 },
            (Ty::Mono(tv1), Ty::Mono(tv2)) => tv1 == tv2,
            _ => false,
        }
    }
}

impl Eq for Ty {}

impl Hash for Ty {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            Ty::Unit => (),
            Ty::Int => (),
            Ty::Bool => (),
            Ty::Arrow(l, r) => {
                l.hash(state);
                r.hash(state);
            }
            Ty::Mono(tv) => tv.hash(state),
        }
    }
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

    pub fn extract_args(&self) -> Vec<Ty> {
        let mut args = Vec::new();
        let mut current = self;

        while let Ty::Arrow(arg, ret) = current {
            args.push(*arg.clone());
            current = ret;
        }

        args
    }

    pub fn mangle(&self) -> String {
        match self {
            Ty::Unit => "unit".to_string(),
            Ty::Int => "int".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Mono(var) => format!("tv{}", var),
            Ty::Arrow(left, right) => {
                let left_mangled = left.mangle();
                let right_mangled = right.mangle();
                format!("fn_{}_to_{}_nf", left_mangled, right_mangled)
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
    mapping: HashMap<NameIdentifier, Scheme>
}

impl TypingContext {
    pub fn new() -> Self {
        TypingContext {
            mapping: HashMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.mapping.is_empty()
    }

    pub fn get_mapping(&self) -> &HashMap<NameIdentifier, Scheme> {
        &self.mapping
    }

    pub fn insert(&mut self, id: NameIdentifier, scheme: Scheme) {
        self.mapping.insert(id, scheme);
    }

    pub fn get(&self, id: &NameIdentifier) -> Option<&Scheme> {
        self.mapping.get(id)
    }

    pub fn contains(&self, id: &NameIdentifier) -> bool {
        self.mapping.contains_key(id)
    }
}