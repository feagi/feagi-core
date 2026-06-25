// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Bluetooth Low Energy (BLE) Hardware Abstraction Layer
//!
//! This module defines the platform-agnostic trait for BLE functionality.
//! Platform implementations (ESP32, nRF52, STM32WB) must implement this trait
//! to provide BLE capabilities.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │ Application (firmware)                       │
//! └─────────────────┬────────────────────────────┘
//!                   │ uses
//! ┌─────────────────▼────────────────────────────┐
//! │ BluetoothProvider trait (THIS FILE)          │
//! │ - start_advertising()                        │
//! │ - is_connected()                             │
//! │ - send() / receive()                         │
//! └─────────────────┬────────────────────────────┘
//!                   │ implements
//! ┌─────────────────▼────────────────────────────┐
//! │ Platform Implementation                      │
//! │ - Esp32Bluetooth (esp-idf BLE)              │
//! │ - Nrf52Bluetooth (TrouBLE/nrf-softdevice)   │
//! │ - Stm32wbBluetooth (ST BLE stack)           │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use feagi_hal::hal::BluetoothProvider;
//! # use feagi_hal::platforms::Esp32Bluetooth;
//!
//! // Platform layer provides the implementation
//! let mut ble: Esp32Bluetooth = /* platform init */;
//!
//! // Start advertising
//! ble.start_advertising("FEAGI-robot").unwrap();
//!
//! // Wait for connection
//! while !ble.is_connected() {
//!     // Poll or sleep
//! }
//!
//! // Send/receive data
//! ble.send(b"Hello FEAGI").unwrap();
//! let mut buf = [0u8; 64];
//! if let Ok(len) = ble.receive(&mut buf) {
//!     // Process received data
//! }
//! ```

/// Connection status for BLE
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectionStatus {
    /// Not connected, not advertising
    Disconnected,
    /// Advertising, waiting for connection
    Advertising,
    /// Connected to a client
    Connected,
    /// Error state (e.g., init failed, advertising failed)
    Error,
}

/// Bluetooth Low Energy provider trait
///
/// This trait must be implemented by each platform to provide BLE capabilities.
/// The trait is designed to be simple and compatible with both async and sync
/// implementations.
///
/// ## Design Principles
///
/// 1. **Minimal API**: Only essential operations
/// 2. **Error transparency**: Platform errors are exposed
/// 3. **No callbacks**: Use polling or async/await at the platform level
/// 4. **Buffer-based I/O**: Caller manages buffers
///
/// ## Thread Safety
///
/// Implementations do NOT need to be `Send` or `Sync` - embedded BLE
/// typically runs in a single executor/thread.
pub trait BluetoothProvider {
    /// Platform-specific error type
    type Error: core::fmt::Debug;

    /// Start BLE advertising with the given device name
    ///
    /// This should set up the BLE stack (if not already initialized) and
    /// begin advertising. The device becomes discoverable with the given name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - BLE stack initialization fails
    /// - Device name is invalid (e.g., too long)
    /// - Advertising setup fails
    ///
    /// # Blocking Behavior
    ///
    /// This method MAY block until advertising is successfully started.
    /// On some platforms (e.g., TrouBLE), this might block until a connection
    /// is made. Check platform documentation for blocking behavior.
    fn start_advertising(&mut self, device_name: &str) -> Result<(), Self::Error>;

    /// Stop BLE advertising
    ///
    /// If already connected, this may also disconnect.
    fn stop_advertising(&mut self) -> Result<(), Self::Error>;

    /// Check if BLE is currently connected to a client
    ///
    /// Returns `true` if a client is connected and data can be exchanged.
    fn is_connected(&self) -> bool;

    /// Get current connection status
    ///
    /// Provides more detail than `is_connected()`.
    fn connection_status(&self) -> ConnectionStatus;

    /// Send data over BLE to the connected client
    ///
    /// On most platforms, this uses the TX characteristic (Notify).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not connected (`is_connected() == false`)
    /// - Data is too large for MTU
    /// - BLE stack error
    ///
    /// # Blocking Behavior
    ///
    /// This method MAY block until the data is queued for transmission.
    /// It does NOT wait for acknowledgment from the client.
    fn send(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Receive data from the connected client
    ///
    /// On most platforms, this reads from the RX characteristic (Write).
    ///
    /// # Returns
    ///
    /// - `Ok(n)` where `n` is the number of bytes written to `buffer`
    /// - `Err(e)` if no data available or BLE error
    ///
    /// # Non-blocking
    ///
    /// This method SHOULD NOT block. If no data is available, return
    /// an error or `Ok(0)`.
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Flush any pending transmit data
    ///
    /// This is optional and may be a no-op on some platforms.
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(()) // Default: no-op
    }
}

/// Helper trait for platforms that support async BLE operations
///
/// This is optional and only used by platforms with async runtimes (embassy, tokio, etc.).
#[cfg(feature = "async")]
pub trait AsyncBluetoothProvider: BluetoothProvider {
    /// Async version of `start_advertising`
    async fn start_advertising_async(&mut self, device_name: &str) -> Result<(), Self::Error>;

    /// Async version of `send`
    async fn send_async(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Async version of `receive`
    async fn receive_async(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock BLE implementation for testing
    struct MockBluetooth {
        connected: bool,
        send_buffer: heapless::Vec<u8, 256>,
        receive_buffer: heapless::Vec<u8, 256>,
    }

    impl MockBluetooth {
        fn new() -> Self {
            Self {
                connected: false,
                send_buffer: heapless::Vec::new(),
                receive_buffer: heapless::Vec::new(),
            }
        }
    }

    impl BluetoothProvider for MockBluetooth {
        type Error = &'static str;

        fn start_advertising(&mut self, _device_name: &str) -> Result<(), Self::Error> {
            self.connected = false;
            Ok(())
        }

        fn stop_advertising(&mut self) -> Result<(), Self::Error> {
            self.connected = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn connection_status(&self) -> ConnectionStatus {
            if self.connected {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            }
        }

        fn send(&mut self, data: &[u8]) -> Result<(), Self::Error> {
            if !self.connected {
                return Err("Not connected");
            }
            self.send_buffer.clear();
            for &byte in data {
                self.send_buffer
                    .push(byte)
                    .map_err(|_| "Send buffer full")?;
            }
            Ok(())
        }

        fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err("Not connected");
            }
            let len = self.receive_buffer.len().min(buffer.len());
            buffer[..len].copy_from_slice(&self.receive_buffer[..len]);
            self.receive_buffer.clear();
            Ok(len)
        }
    }

    #[test]
    fn test_mock_bluetooth_advertising() {
        let mut ble = MockBluetooth::new();
        assert!(!ble.is_connected());
        assert!(ble.start_advertising("test").is_ok());
    }

    #[test]
    fn test_mock_bluetooth_send_not_connected() {
        let mut ble = MockBluetooth::new();
        assert!(ble.send(b"test").is_err());
    }

    #[test]
    fn test_mock_bluetooth_send_connected() {
        let mut ble = MockBluetooth::new();
        ble.connected = true; // Simulate connection
        assert!(ble.send(b"test").is_ok());
        assert_eq!(ble.send_buffer.as_slice(), b"test");
    }

    #[test]
    fn test_mock_bluetooth_receive_not_connected() {
        let mut ble = MockBluetooth::new();
        let mut buf = [0u8; 16];
        assert!(ble.receive(&mut buf).is_err());
    }

    #[test]
    fn test_mock_bluetooth_receive_connected() {
        let mut ble = MockBluetooth::new();
        ble.connected = true;
        ble.receive_buffer.extend_from_slice(b"data").unwrap();

        let mut buf = [0u8; 16];
        let len = ble.receive(&mut buf).unwrap();
        assert_eq!(len, 4);
        assert_eq!(&buf[..len], b"data");
    }
}
