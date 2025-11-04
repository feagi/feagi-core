#![no_std]
#![warn(missing_docs)]

//! # FEAGI Embedded
//! 
//! Platform abstraction and implementations for FEAGI embedded neural networks.
//! 
//! This crate provides:
//! - **HAL traits** (`hal` module) - Platform-agnostic hardware abstractions
//! - **Platform implementations** (`platforms` module) - Concrete implementations for ESP32, Arduino, STM32, etc.
//! 
//! ## Usage
//! 
//! ### Advanced Users (Direct HAL Usage)
//! ```no_run
//! use feagi_embedded::prelude::*;
//! use feagi_runtime_embedded::{NeuronArray, SynapseArray};
//! use feagi_types::INT8Value;
//! 
//! fn main() -> ! {
//!     let platform = Esp32Platform::init().expect("Failed to init");
//!     let mut neurons = NeuronArray::<INT8Value, 1000>::new();
//!     let mut synapses = SynapseArray::<5000>::new();
//!     
//!     // Custom network topology
//!     // Custom burst loop
//!     loop {
//!         neurons.process_burst(&synapses);
//!     }
//! }
//! ```
//! 
//! ### SDK Users
//! See `feagi-nano` crate for high-level SDK with NetworkBuilder, templates, etc.
//! 
//! ## Feature Flags
//! 
//! Platforms are selected via feature flags:
//! - `esp32` - ESP32, ESP32-S3, ESP32-C3 support
//! - `arduino-due` - Arduino Due support (future)
//! - `stm32f4` - STM32F4 series support (future)
//! - `hailo` - Hailo-8 neural accelerator support (future)

// HAL trait definitions
pub mod hal;

// Platform implementations
pub mod platforms;

// Re-export commonly used types
pub use hal::{
    Platform, 
    TimeProvider, 
    SerialIO, 
    GpioProvider, 
    Logger, 
    LogLevel,
    NeuralAccelerator,
    AcceleratorCapabilities,
};

// Re-export platform implementations
#[cfg(feature = "esp32")]
pub use platforms::Esp32Platform;

// Re-export core FEAGI types for convenience
pub use feagi_types::{INT8Value, NeuralValue};
pub use feagi_runtime_embedded::{NeuronArray, SynapseArray};

/// Prelude module for convenient imports
/// 
/// ```no_run
/// use feagi_embedded::prelude::*;
/// ```
pub mod prelude {
    pub use crate::hal::*;
    pub use crate::platforms::*;
    pub use feagi_types::{INT8Value, NeuralValue};
    pub use feagi_runtime_embedded::{NeuronArray, SynapseArray};
    
    #[cfg(feature = "esp32")]
    pub use crate::platforms::Esp32Platform;
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get library version
pub fn version() -> &'static str {
    VERSION
}

