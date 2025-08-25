//! Neural decoder implementations for converting XYZP activations to I/O data.
//!
//! This module contains implementations of the `NeuronXYZPDecoder` trait for
//! converting neural activation patterns back into meaningful external data types.
//! Decoders analyze spatial patterns of neural activity and extract values that
//! can be used for motor control, data output, or further processing.
//!
//! # Decoder Architecture
//!
//! Decoders reverse the encoding process by:
//! 1. Reading neural activation patterns from cortical areas
//! 2. Analyzing spatial distributions of activation
//! 3. Extracting meaningful values using algorithm-specific methods
//! 4. Converting to appropriate I/O data types
//!
//! # Implementation Status
//!
//! Currently, decoder implementations are in development. The module structure
//! is prepared for future decoder types that will complement the existing
//! encoder implementations.
//!
//! # Future Decoders
//!
//! Planned decoder implementations include:
//! - **Linear decoders**: For extracting continuous values from activation patterns
//! - **Bidirectional decoders**: For signed value extraction
//! - **Visual decoders**: For reconstructing spatial data from neural patterns
//! - **Multi-region decoders**: For aggregating information across cortical areas
//!
//! # Thread Safety
//! All future decoder implementations will be designed to be thread-safe
//! for use in multi-threaded neural simulation environments.

//mod linear_normalized_floats;

//pub use linear_normalized_floats::*;