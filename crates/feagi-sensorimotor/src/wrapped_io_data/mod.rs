//! Type-safe wrappers for heterogeneous I/O data.
//!
//! Provides enums that can hold various data types (percentages, images, etc.)
//! in a type-safe manner, enabling dynamic dispatch for I/O operations while
//! maintaining compile-time type checking where possible.
//!
//! These types are tightly integrated with the neuron encoding/decoding system.

mod wrapped_io_data;
mod wrapped_io_type;

pub use wrapped_io_data::WrappedIOData;
pub use wrapped_io_type::WrappedIOType;
