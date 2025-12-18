// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/// Neural accelerator control traits.
pub mod accelerator;
/// Bluetooth Low Energy communication traits.
pub mod bluetooth;
/// General-purpose I/O abstractions for pins.
pub mod gpio;
/// Logging interfaces for embedded targets.
pub mod logger;
/// Serial input/output traits for UART-style communication.
pub mod serial;
/// Hardware Abstraction Layer (HAL) trait definitions for embedded platforms
///
/// This module defines platform-agnostic traits that must be implemented
/// by each platform to provide:
/// - Time management (TimeProvider)
/// - Serial I/O (SerialIO)
/// - GPIO control (GpioProvider)
/// - Logging (Logger)
/// - Neural acceleration (NeuralAccelerator)

/// Timekeeping abstractions (monotonic timers, delays).
pub mod time;
/// USB CDC Serial communication traits.
pub mod usb_cdc;

// Re-export trait types
pub use accelerator::{AcceleratorCapabilities, NeuralAccelerator};
pub use bluetooth::{BluetoothProvider, ConnectionStatus};
pub use gpio::GpioProvider;
pub use logger::{LogLevel, Logger};
pub use serial::SerialIO;
pub use time::TimeProvider;
pub use usb_cdc::{UsbCdcProvider, UsbConnectionStatus};

#[cfg(feature = "async")]
pub use bluetooth::AsyncBluetoothProvider;

#[cfg(feature = "async")]
pub use usb_cdc::AsyncUsbCdcProvider;

/// Convenience trait combining common platform capabilities
///
/// Most embedded platforms will implement this trait by combining
/// TimeProvider, SerialIO, GpioProvider, and Logger.
pub trait Platform: TimeProvider + Logger {
    /// Get platform name (e.g., "ESP32", "Arduino Due", "STM32F4")
    fn name(&self) -> &'static str;

    /// Get CPU frequency in Hz
    fn cpu_frequency_hz(&self) -> u32;

    /// Get available memory in bytes
    fn available_memory_bytes(&self) -> usize;

    /// Get platform uptime in milliseconds
    fn uptime_ms(&self) -> u64 {
        self.get_time_us() / 1000
    }
}
