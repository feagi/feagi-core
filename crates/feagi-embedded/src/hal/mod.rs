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
/// Serial input/output traits for UART-style communication.
pub mod serial;
/// General-purpose I/O abstractions for pins.
pub mod gpio;
/// Logging interfaces for embedded targets.
pub mod logger;
/// Neural accelerator control traits.
pub mod accelerator;
/// Bluetooth Low Energy communication traits.
pub mod bluetooth;
/// USB CDC Serial communication traits.
pub mod usb_cdc;

// Re-export trait types
pub use time::TimeProvider;
pub use serial::SerialIO;
pub use gpio::GpioProvider;
pub use logger::{Logger, LogLevel};
pub use accelerator::{NeuralAccelerator, AcceleratorCapabilities};
pub use bluetooth::{BluetoothProvider, ConnectionStatus};
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

