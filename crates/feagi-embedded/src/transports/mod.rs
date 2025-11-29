//! Transport Layer for FEAGI Communication
//!
//! This module provides transport-agnostic protocol handling for FEAGI embodiments.
//! The same protocol layer works with multiple transport mechanisms:
//!
//! - **Bluetooth LE** (wireless, 2 Mbps, 10-100m range)
//! - **USB CDC Serial** (wired, 12 Mbps, 5m cable)
//! - **UART** (wired, 115200 baud typical)
//! - **WiFi** (future)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  FEAGI Core (Python/Rust)              │
//! └─────────────────┬───────────────────────┘
//!                   │ sends commands
//! ┌─────────────────▼───────────────────────┐
//! │  Transport Layer (BLE/USB/UART/WiFi)    │
//! │  - BLE: BluetoothProvider               │
//! │  - USB: UsbCdcProvider                  │
//! │  - UART: SerialIO                       │
//! └─────────────────┬───────────────────────┘
//!                   │ raw bytes
//! ┌─────────────────▼───────────────────────┐
//! │  Protocol Layer (THIS MODULE)           │
//! │  - Parses binary packets                │
//! │  - Extracts commands                    │
//! │  - Formats responses                    │
//! └─────────────────┬───────────────────────┘
//!                   │ Commands
//! ┌─────────────────▼───────────────────────┐
//! │  Application (Embodiment Firmware)      │
//! │  - Updates LED matrix                   │
//! │  - Controls GPIO                        │
//! │  - Reads sensors                        │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Key Insight: Transport Independence
//!
//! The protocol layer doesn't care if bytes arrive via:
//! - Bluetooth notifications
//! - USB CDC bulk transfers  
//! - UART RX interrupts
//! - WiFi TCP packets
//!
//! **It only cares about the byte format.**
//!
//! ## Usage with BLE
//!
//! ```rust,no_run
//! use feagi_embedded::transports::protocol::Protocol;
//! use feagi_embedded::hal::BluetoothProvider;
//! # use feagi_embedded::platforms::Esp32Bluetooth;
//!
//! let mut ble: Esp32Bluetooth = /* init */;
//! let mut protocol = Protocol::new("FEAGI-robot");
//!
//! loop {
//!     // Receive bytes from BLE
//!     let mut buf = [0u8; 64];
//!     if let Ok(len) = ble.receive(&mut buf) {
//!         // Protocol layer parses bytes
//!         protocol.process_received_data(&buf[..len]);
//!     }
//!     
//!     // Protocol layer extracts commands
//!     if let Some(cmd) = protocol.receive_command() {
//!         // Handle command (GPIO, LED, etc.)
//!     }
//! }
//! ```
//!
//! ## Usage with USB CDC
//!
//! ```rust,no_run
//! use feagi_embedded::transports::protocol::Protocol;
//! use feagi_embedded::hal::UsbCdcProvider;
//! # use feagi_embedded::platforms::Nrf52UsbCdc;
//!
//! let mut usb: Nrf52UsbCdc = /* init */;
//! let mut protocol = Protocol::new("FEAGI-robot");
//!
//! loop {
//!     // Receive bytes from USB
//!     let mut buf = [0u8; 64];
//!     if let Ok(len) = usb.read(&mut buf) {
//!         // SAME protocol layer parses bytes
//!         protocol.process_received_data(&buf[..len]);
//!     }
//!     
//!     // SAME command extraction
//!     if let Some(cmd) = protocol.receive_command() {
//!         // Handle command (GPIO, LED, etc.)
//!     }
//! }
//! ```
//!
//! **Notice:** The protocol handling is identical. Only the transport API differs
//! (`ble.receive()` vs `usb.read()`).

pub mod protocol;

// Re-export for convenience
pub use protocol::{Protocol, Command, PacketCommand};

