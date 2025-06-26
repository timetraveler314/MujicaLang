use crate::frontend::ast::ASTAtom;
use crate::frontend::name_resolution::ResolvedIdent;

pub mod uncurry;
pub mod conversion;
pub mod knf;
pub mod anf;

#[derive(Debug)]
pub enum CoreError {
    ConversionError(String), // Represents an error during conversion
}

pub type Atom = ASTAtom<ResolvedIdent>;