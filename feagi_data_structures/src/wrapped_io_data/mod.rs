/// Enums that describe or wrap values for use in IO operations.
/// These must be in this crate due to the encoders / decoders

mod wrapped_io_type;
mod wrapped_io_data;

pub use wrapped_io_type::WrappedIOType;
pub use wrapped_io_data::WrappedIOData;