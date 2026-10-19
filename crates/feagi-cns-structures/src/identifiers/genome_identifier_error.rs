use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenomeIdentifierError {
    #[error("Unable to parse {0} ID from Bytes!")]
    CannotParseFromBytes(&'static str),
    #[error("Invalid Faux Identifier struct! {0}")]
    InvalidFauxStruct(&'static str),
}
