//! Bluetooth Low Energy (BLE) support for FEAGI embedded systems
//!
//! This module provides a platform-agnostic BLE abstraction layer for embedded
//! FEAGI agents. It includes:
//! - Protocol layer for FEAGI commands and neuron data
//! - Nordic UART Service (NUS) definitions
//! - Platform-specific BLE stack implementations
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Application (embodiment firmware)      │
//! └─────────────────┬───────────────────────┘
//!                   │ uses
//! ┌─────────────────▼───────────────────────┐
//! │  Protocol Layer (protocol.rs)           │
//! │  - Command parsing                      │
//! │  - Neuron data formatting               │
//! │  - JSON message handling                │
//! └─────────────────┬───────────────────────┘
//!                   │ uses
//! ┌─────────────────▼───────────────────────┐
//! │  BLE HAL Trait (../hal/bluetooth.rs)    │
//! └─────────────────┬───────────────────────┘
//!                   │ implements
//! ┌─────────────────▼───────────────────────┐
//! │  Platform Implementation                │
//! │  - nRF52 (TrouBLE)                     │
//! │  - ESP32 (esp-idf)                     │
//! │  - STM32WB (ST BLE stack)              │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use feagi_embedded::bluetooth::protocol::BluetoothService;
//! use feagi_embedded::bluetooth::nus::FEAGI_DEVICE_NAME;
//!
//! // Initialize BLE stack (platform-specific)
//! # #[cfg(feature = "bluetooth-esp32")]
//! # {
//! use feagi_embedded::platforms::Esp32Bluetooth;
//! let mut ble = Esp32Bluetooth::new("FEAGI-robot").expect("BLE init failed");
//! # }
//!
//! // Protocol layer is platform-agnostic
//! let mut service = BluetoothService::new("FEAGI-robot");
//!
//! loop {
//!     // Receive commands from FEAGI
//!     # #[cfg(feature = "bluetooth-esp32")]
//!     # {
//!     if let Ok(data) = ble.receive_data() {
//!         service.process_received_data(&data);
//!     }
//!
//!     // Check for parsed commands
//!     if let Some(cmd) = service.receive_command() {
//!         // Handle command (GPIO, LED, etc.)
//!     }
//!     # }
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `bluetooth-nrf52` - nRF52-based devices (micro:bit, nRF52840-DK) using TrouBLE
//! - `bluetooth-esp32` - ESP32/ESP32-S3/ESP32-C3 using esp-idf BLE stack
//! - `bluetooth-stm32wb` - STM32WB series with built-in BLE
//!
//! ## Current Status
//!
//! | Platform | Status | Notes |
//! |----------|--------|-------|
//! | nRF52 (TrouBLE) | ⚠️ Experimental | Executor compatibility issues |
//! | ESP32 (esp-idf) | 🟢 Planned | Should work with embassy |
//! | STM32WB | 🔵 Planned | Hardware BLE controller |

pub mod nus;

// Re-export the shared transport protocol for backward compatibility
// The protocol is now in the transports module since it works with both BLE and USB
pub use crate::transports::protocol::{Protocol as BluetoothProtocol, Command, PacketCommand};

// Platform-specific implementations are in src/platforms/
// and enabled via feature flags

