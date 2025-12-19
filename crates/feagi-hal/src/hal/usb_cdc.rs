// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! USB CDC (Communications Device Class) Hardware Abstraction Layer
//!
//! This module defines the platform-agnostic trait for USB CDC Serial functionality.
//! Platform implementations (ESP32, nRF52, STM32, RP2040) must implement this trait
//! to provide USB serial capabilities.
//!
//! ## What is USB CDC?
//!
//! USB CDC makes the device appear as a virtual serial port (e.g., `/dev/ttyACM0`, `COM3`).
//! It's a standard USB protocol that works with any OS without custom drivers.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────┐
//! │ Application (firmware)                       │
//! └─────────────────┬────────────────────────────┘
//!                   │ uses
//! ┌─────────────────▼────────────────────────────┐
//! │ UsbCdcProvider trait (THIS FILE)             │
//! │ - init()                                     │
//! │ - write() / read()                           │
//! │ - is_connected()                             │
//! └─────────────────┬────────────────────────────┘
//!                   │ implements
//! ┌─────────────────▼────────────────────────────┐
//! │ Platform Implementation                      │
//! │ - Nrf52UsbCdc (embassy-nrf USB)             │
//! │ - Esp32UsbCdc (esp-idf-hal USB)             │
//! │ - Stm32UsbCdc (stm32-usbd)                  │
//! │ - Rp2040UsbCdc (rp2040-hal USB)             │
//! └──────────────────────────────────────────────┘
//! ```
//!
//! ## Comparison: USB CDC vs BLE
//!
//! | Feature | USB CDC | BLE |
//! |---------|---------|-----|
//! | **Speed** | 12 Mbps (USB 2.0 Full Speed) | 2 Mbps (BLE 5) |
//! | **Latency** | <1ms | 7.5-30ms (connection interval) |
//! | **Range** | 5m (cable length) | 10-100m (wireless) |
//! | **Pairing** | None (plug & play) | Required |
//! | **Power** | Higher (USB powered) | Lower (BLE optimized) |
//! | **Use Case** | Development, high-speed data | Production, wireless |
//!
//! ## Usage
//!
//! ```rust,no_run
//! use feagi_hal::hal::UsbCdcProvider;
//! # use feagi_hal::platforms::Nrf52UsbCdc;
//!
//! // Platform layer provides the implementation
//! let mut usb: Nrf52UsbCdc = /* platform init */;
//!
//! // Write data
//! usb.write(b"Hello FEAGI\n").unwrap();
//!
//! // Read data
//! let mut buf = [0u8; 64];
//! if let Ok(len) = usb.read(&mut buf) {
//!     // Process received data
//! }
//! ```

/// USB CDC connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UsbConnectionStatus {
    /// USB cable not connected or not enumerated
    Disconnected,
    /// USB enumerated, DTR/RTS signals indicate host is ready
    Connected,
    /// USB suspended (host sleep/power saving)
    Suspended,
}

/// USB CDC Serial provider trait
///
/// This trait must be implemented by each platform to provide USB CDC serial capabilities.
/// The trait follows UART-like semantics: `write()` sends data, `read()` receives data.
///
/// ## Design Principles
///
/// 1. **Simple API**: Mirrors standard UART/serial interfaces
/// 2. **Non-blocking**: `read()` returns immediately if no data
/// 3. **Buffered I/O**: Platform manages TX/RX buffers internally
/// 4. **Connection-aware**: `is_connected()` checks if host is ready
///
/// ## Thread Safety
///
/// Implementations do NOT need to be `Send` or `Sync` - embedded USB
/// typically runs in a single executor/thread.
///
/// ## Flow Control
///
/// USB CDC uses DTR (Data Terminal Ready) and RTS (Request To Send) signals
/// to indicate when the host is ready. `is_connected()` should check these signals.
pub trait UsbCdcProvider {
    /// Platform-specific error type
    type Error: core::fmt::Debug;

    /// Initialize USB CDC serial
    ///
    /// This sets up the USB stack, configures endpoints, and begins enumeration.
    /// After this call, the device should appear as a serial port on the host.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - USB peripheral initialization fails
    /// - USB descriptor configuration is invalid
    /// - Platform USB driver is unavailable
    ///
    /// # Blocking Behavior
    ///
    /// This method MAY block briefly during USB enumeration (~100-500ms).
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Check if USB is connected and host is ready
    ///
    /// Returns `true` if:
    /// - USB cable is plugged in
    /// - Device is enumerated by host
    /// - DTR signal is asserted (host terminal is open)
    ///
    /// Returns `false` if:
    /// - Cable unplugged
    /// - Not enumerated
    /// - Host terminal closed (DTR de-asserted)
    fn is_connected(&self) -> bool;

    /// Get current USB connection status
    ///
    /// Provides more detail than `is_connected()`.
    fn connection_status(&self) -> UsbConnectionStatus;

    /// Write data to USB CDC (send to host)
    ///
    /// Writes data to the TX buffer. Data will be sent to the host in the next
    /// USB IN transfer (typically within 1-10ms).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - TX buffer is full (caller should retry later)
    /// - USB is disconnected
    /// - USB peripheral error
    ///
    /// # Blocking Behavior
    ///
    /// This method SHOULD NOT block. If the TX buffer is full, return an error
    /// immediately. The caller can retry or use `flush()` to wait.
    ///
    /// # Returns
    ///
    /// - `Ok(n)` where `n` is the number of bytes written (may be less than `data.len()`)
    /// - `Err(e)` if write failed
    fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error>;

    /// Read data from USB CDC (receive from host)
    ///
    /// Reads available data from the RX buffer into the provided buffer.
    ///
    /// # Returns
    ///
    /// - `Ok(n)` where `n` is the number of bytes read (0 if no data available)
    /// - `Err(e)` if read failed
    ///
    /// # Non-blocking
    ///
    /// This method MUST NOT block. If no data is available, return `Ok(0)` immediately.
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Flush TX buffer (block until all data is sent)
    ///
    /// Waits until all pending TX data has been sent to the host.
    ///
    /// # Blocking Behavior
    ///
    /// This method MAY block until the TX buffer is empty (typically <10ms).
    ///
    /// # Use Cases
    ///
    /// - Before entering sleep mode
    /// - Before disconnecting USB
    /// - After writing critical data (e.g., error messages)
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Get number of bytes available to read (optional)
    ///
    /// Returns the number of bytes in the RX buffer.
    /// Default implementation returns 0 (unknown).
    fn available(&self) -> usize {
        0 // Default: unknown
    }

    /// Get free space in TX buffer (optional)
    ///
    /// Returns the number of bytes that can be written without blocking.
    /// Default implementation returns 0 (unknown).
    fn write_capacity(&self) -> usize {
        0 // Default: unknown
    }
}

/// Helper trait for platforms that support async USB CDC operations
///
/// This is optional and only used by platforms with async runtimes (embassy, tokio, etc.).
#[cfg(feature = "async")]
pub trait AsyncUsbCdcProvider: UsbCdcProvider {
    /// Async version of `write`
    ///
    /// Waits until data can be written (if TX buffer is full).
    async fn write_async(&mut self, data: &[u8]) -> Result<usize, Self::Error>;

    /// Async version of `read`
    ///
    /// Waits until data is available (blocks if RX buffer is empty).
    async fn read_async(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;

    /// Async version of `flush`
    async fn flush_async(&mut self) -> Result<(), Self::Error>;
}

/// Helper: Write a complete line with newline
///
/// This is a convenience method that ensures a newline is appended.
pub fn writeln<T: UsbCdcProvider>(usb: &mut T, data: &[u8]) -> Result<(), T::Error> {
    usb.write(data)?;
    usb.write(b"\n")?;
    Ok(())
}

/// Helper: Read until newline or buffer full
///
/// Returns the number of bytes read (including newline if present).
pub fn read_line<T: UsbCdcProvider>(usb: &mut T, buffer: &mut [u8]) -> Result<usize, T::Error> {
    let mut total = 0;
    loop {
        let n = usb.read(&mut buffer[total..])?;
        if n == 0 {
            break; // No more data available
        }
        total += n;

        // Check for newline
        if buffer[..total].contains(&b'\n') {
            break;
        }

        // Buffer full
        if total >= buffer.len() {
            break;
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock USB CDC implementation for testing
    struct MockUsbCdc {
        connected: bool,
        tx_buffer: heapless::Vec<u8, 256>,
        rx_buffer: heapless::Vec<u8, 256>,
    }

    impl MockUsbCdc {
        fn new() -> Self {
            Self {
                connected: false,
                tx_buffer: heapless::Vec::new(),
                rx_buffer: heapless::Vec::new(),
            }
        }

        // Helper: Simulate host sending data
        fn _push_rx_data(&mut self, data: &[u8]) {
            for &byte in data {
                let _ = self.rx_buffer.push(byte);
            }
        }
    }

    impl UsbCdcProvider for MockUsbCdc {
        type Error = &'static str;

        fn init(&mut self) -> Result<(), Self::Error> {
            self.connected = false; // Not connected until enumerated
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn connection_status(&self) -> UsbConnectionStatus {
            if self.connected {
                UsbConnectionStatus::Connected
            } else {
                UsbConnectionStatus::Disconnected
            }
        }

        fn write(&mut self, data: &[u8]) -> Result<usize, Self::Error> {
            if !self.connected {
                return Err("Not connected");
            }

            let mut written = 0;
            for &byte in data {
                if self.tx_buffer.push(byte).is_ok() {
                    written += 1;
                } else {
                    break; // Buffer full
                }
            }
            Ok(written)
        }

        fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
            let len = self.rx_buffer.len().min(buffer.len());
            buffer[..len].copy_from_slice(&self.rx_buffer[..len]);

            // Remove read bytes from RX buffer
            for _ in 0..len {
                self.rx_buffer.remove(0);
            }

            Ok(len)
        }

        fn flush(&mut self) -> Result<(), Self::Error> {
            if !self.connected {
                return Err("Not connected");
            }
            self.tx_buffer.clear(); // Simulate data sent
            Ok(())
        }

        fn available(&self) -> usize {
            self.rx_buffer.len()
        }

        fn write_capacity(&self) -> usize {
            self.tx_buffer.capacity() - self.tx_buffer.len()
        }
    }

    #[test]
    fn test_mock_usb_init() {
        let mut usb = MockUsbCdc::new();
        assert!(usb.init().is_ok());
        assert!(!usb.is_connected());
    }

    #[test]
    fn test_mock_usb_write_not_connected() {
        let mut usb = MockUsbCdc::new();
        assert!(usb.write(b"test").is_err());
    }

    #[test]
    fn test_mock_usb_write_connected() {
        let mut usb = MockUsbCdc::new();
        usb.connected = true;

        let written = usb.write(b"Hello").unwrap();
        assert_eq!(written, 5);
        assert_eq!(usb.tx_buffer.as_slice(), b"Hello");
    }

    #[test]
    fn test_mock_usb_read() {
        let mut usb = MockUsbCdc::new();
        usb._push_rx_data(b"Data from host");

        let mut buf = [0u8; 64];
        let len = usb.read(&mut buf).unwrap();
        assert_eq!(len, 14);
        assert_eq!(&buf[..len], b"Data from host");
    }

    #[test]
    fn test_mock_usb_flush() {
        let mut usb = MockUsbCdc::new();
        usb.connected = true;
        usb.write(b"test").unwrap();

        assert!(!usb.tx_buffer.is_empty());
        usb.flush().unwrap();
        assert!(usb.tx_buffer.is_empty());
    }

    #[test]
    fn test_writeln_helper() {
        let mut usb = MockUsbCdc::new();
        usb.connected = true;

        writeln(&mut usb, b"Line 1").unwrap();
        assert_eq!(usb.tx_buffer.as_slice(), b"Line 1\n");
    }

    #[test]
    fn test_connection_status() {
        let usb = MockUsbCdc::new();
        assert_eq!(usb.connection_status(), UsbConnectionStatus::Disconnected);
    }

    #[test]
    fn test_available_and_capacity() {
        let mut usb = MockUsbCdc::new();
        usb._push_rx_data(b"test");

        assert_eq!(usb.available(), 4);
        assert_eq!(usb.write_capacity(), 256); // Empty TX buffer
    }
}
