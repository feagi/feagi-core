//! Not everything can be done with traits, a lot must be done through enums and structs.
//! To make things simpler, these per model / model quantization enums are defined under here as
//! they will be created at compile time given the set of models statically defined
//!
//! There are 2 common subclasses of enums. Nested enums are just that, enums that have variants
//! that have other enums. Convenient to work with within Rust, but not very portable across
//! device interfaces. Packed enums represent the same data as Nested enums, but within a single
//! flat enum that fits within a single byte. Not as practical to work with but since it is
//! effectively just a u8 (or less), considered universal. Both implementations may be defined for
//! the same piece of information, and could be converted between as needed

// TODO build.rs should generate these enums and structs

pub mod cortical_layout;
pub mod model_type_and_quantization;
pub mod cortical_writer_by_model_quant;