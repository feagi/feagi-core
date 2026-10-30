

#[derive(Debug, thiserror::Error)]
pub enum CorticalMatchSelectionError {
    #[error("Tried to access this cortical supertype as a supertype {tried} when it was actually a supertype {actual}")]
    IncorrectCorticalSuperType{tried: &'static str, actual: &'static str},
    #[error("Tried to access this cortical type as a type {tried} when it was actually a type {actual}")]
    IncorrectCorticalType{tried: &'static str, actual: &'static str},
    #[error("Tried to access this cortical type as a type {tried} when it was actually a type {actual}")]
    IncorrectCorticalModel{tried: &'static str, actual: &'static str},

    // TODO incorrect quant transparency
}