use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenomeIdentifierError {
    #[error("Unable to parse {0} ID from URL!")]
    CannotParseURLParameter(&'static str)
}




