pub mod uncurry;
mod conversion;
mod knf;

#[derive(Debug)]
pub enum CoreError {
    ConversionError(String), // Represents an error during conversion
}