//! Quantization describes how many bits (and how) to allocate  to various variables. Higher
//! quantization results in higher memory usage, but results in higher precision / higher index
//! ranging.
//!
//! Broadly speaking, there are 2 classes of Quantization levels:
//! NPU Level: Defines a quantization for a data type that affects the entire NPU. This is used for
//! indexing and other types where converting back and forth is too big a hassle with little
//! size benefit
//!
//! Structure Level: Can be set per structure (such as per cortical area), where possible
//! space savings are worth the increased complexity
mod unsigned_integer;
mod signed_integer;
mod nonzero_count;
mod percentage;
mod value;
pub mod spatial;
mod quantization;
mod shared;

pub use unsigned_integer::QuantizableUIntType;
pub use signed_integer::QuantizableIntType;
pub use nonzero_count::QuantizableNonzeroUIntType;
pub use value::QuantizableValueType;
pub use percentage::QuantizablePercentType;
pub use quantization::QuantizationLevel;
pub use shared::{FeagiBaseSingleElementQuantizationType, FeagiBaseQuantizationType, FeagiBaseMultiElementQuantizationType};
