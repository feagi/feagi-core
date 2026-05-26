use crate::quantizable::base_types::QuantizedElementBase;

/// Root Quantizable indicator trait. Simply states that the type of data can be quantized to
/// different sizes. States nothing more! You are trusted in this case that the type is
/// Quantizable!
pub trait FeagiQuantizedGeneric {}


//region Base Rust type implementations



//endregion

/// Enforces that the quantized value is of a singular class that we have quantized forms of
pub trait FeagiQuantizedElement<Q>: FeagiQuantizedGeneric
where
    Q: QuantizedElementBase,
{ }