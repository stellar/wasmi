use crate::engine::TranslationError;
use core::{
    fmt,
    fmt::{Debug, Display},
};
use wasmparser::BinaryReaderError as ParserError;

/// Errors that may occur upon reading, parsing and translating Wasm modules.
#[derive(Debug)]
pub enum ModuleError {
    /// Encountered when there is a Wasm parsing error.
    Parser(ParserError),
    /// Encountered when there is a Wasm to `wasmi` translation error.
    Translation(TranslationError),
    /// Encountered when a Wasm section is malformed in a way that is cheap to
    /// detect up front, before handing it to the validator.
    Malformed(&'static str),
}

impl Display for ModuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleError::Parser(error) => Display::fmt(error, f),
            ModuleError::Translation(error) => Display::fmt(error, f),
            ModuleError::Malformed(reason) => write!(f, "malformed Wasm module: {reason}"),
        }
    }
}

impl From<ParserError> for ModuleError {
    fn from(error: ParserError) -> Self {
        Self::Parser(error)
    }
}

impl From<TranslationError> for ModuleError {
    fn from(error: TranslationError) -> Self {
        Self::Translation(error)
    }
}
