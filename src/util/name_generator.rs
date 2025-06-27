use crate::frontend::name_resolution::{NameIdentifier, ResolvedIdent};

#[derive(Debug)]
pub struct NameGenerator {
    prefix: String,
    counter: usize,
}

impl NameGenerator {
    pub fn new(prefix: &str) -> Self {
        NameGenerator {
            prefix: prefix.to_string(),
            counter: 0,
        }
    }

    pub fn next_name(&mut self) -> String {
        let name = format!("{}{}", self.prefix, self.counter);
        self.counter += 1;
        name
    }
    
    pub fn fresh_ident(&mut self) -> ResolvedIdent {
        let name = self.next_name();
        
        ResolvedIdent {
            name: name.clone(),
            id: NameIdentifier(name),
        }
    }
}