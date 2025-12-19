// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

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

/// Hardware abstraction traits shared by all platforms.
pub mod hal;

/// Concrete platform implementations (ESP32, STM32, etc.).
pub mod platforms;

/// Bluetooth Low Energy protocol and abstractions
pub mod bluetooth;

/// Transport layer (protocol that works with BLE, USB CDC, UART, etc.)
pub mod transports;

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
    BluetoothProvider,
    ConnectionStatus,
    UsbCdcProvider,
    UsbConnectionStatus,
};

#[cfg(feature = "async")]
pub use hal::{AsyncBluetoothProvider, AsyncUsbCdcProvider};

// Re-export transport protocol
pub use transports::{Protocol, Command, PacketCommand};

// Re-export platform implementations
#[cfg(feature = "esp32")]
pub use platforms::Esp32Platform;

#[cfg(feature = "arduino-due")]
pub use platforms::ArduinoDuePlatform;

#[cfg(feature = "stm32f4")]
pub use platforms::Stm32F4Platform;

#[cfg(feature = "rpi-pico")]
pub use platforms::RpiPicoPlatform;

#[cfg(feature = "hailo")]
pub use platforms::{Hailo8Accelerator, HailoError, HybridCpuHailo};

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
    
    #[cfg(feature = "arduino-due")]
    pub use crate::platforms::ArduinoDuePlatform;
    
    #[cfg(feature = "stm32f4")]
    pub use crate::platforms::Stm32F4Platform;
    
    #[cfg(feature = "rpi-pico")]
    pub use crate::platforms::RpiPicoPlatform;
    
    #[cfg(feature = "hailo")]
    pub use crate::platforms::{Hailo8Accelerator, HailoError, HybridCpuHailo};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get library version
pub fn version() -> &'static str {
    VERSION
}

