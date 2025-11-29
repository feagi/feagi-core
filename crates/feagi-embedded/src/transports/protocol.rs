//! FEAGI Transport Protocol (Transport-Agnostic)
//!
//! This module defines the binary protocol for FEAGI→Embodiment communication.
//! It works with **any transport layer**: BLE, USB CDC, UART, WiFi, etc.
//!
//! The protocol layer is responsible for:
//! - Parsing binary packets into structured commands
//! - Buffering incomplete packets
//! - Formatting responses (sensor data, capabilities)
//!
//! The protocol layer does NOT handle:
//! - Transport connection/disconnection
//! - Sending/receiving bytes (that's the transport's job)
//! - Platform-specific hardware access
//!
//! ## Binary Protocol Format
//!
//! ```text
//! ┌──────────┬──────────┬────────────────────┐
//! │ Command  │ Length   │ Payload            │
//! │ (1 byte) │ (1 byte) │ (variable)         │
//! └──────────┴──────────┴────────────────────┘
//! ```
//!
//! ### Command Types
//!
//! | ID | Name | Payload | Description |
//! |----|------|---------|-------------|
//! | 0x01 | NeuronFiring | count + (x,y) pairs | Fired neurons for display |
//! | 0x02 | SetGpio | pin + value | Digital GPIO control |
//! | 0x03 | SetPwm | pin + duty | PWM control (0-255) |
//! | 0x04 | SetLedMatrix | 25 bytes | Full 5×5 LED matrix |
//! | 0x05 | GetCapabilities | none | Request device info |
//!
//! ## Example: Neuron Firing Packet
//!
//! To light LEDs at (1,2) and (3,4):
//!
//! ```text
//! [0x01] [0x02] [0x01, 0x02, 0x03, 0x04]
//!   │      │      └─ Coordinates: (1,2), (3,4)
//!   │      └─ Count: 2 neurons
//!   └─ Command: NeuronFiring (0x01)
//! ```
//!
//! ## Transport Independence
//!
//! This protocol works identically over:
//! - **BLE**: Bytes arrive via GATT Write characteristic
//! - **USB CDC**: Bytes arrive via USB bulk OUT endpoint
//! - **UART**: Bytes arrive via serial RX interrupt
//! - **WiFi**: Bytes arrive via TCP socket
//!
//! The protocol doesn't know or care which transport is used!

#![no_std]

use heapless::Vec;

/// FEAGI commands (parsed from binary packets)
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    /// Set a GPIO pin to high or low
    SetGpio { pin: u8, value: bool },
    /// Set PWM duty cycle (0-255) on a pin
    SetPwm { pin: u8, duty: u8 },
    /// Set full LED matrix (5x5 = 25 bytes, brightness 0-255)
    SetLedMatrix { data: [u8; 25] },
    /// Neuron firing coordinates for LED matrix visualization
    /// Each coordinate is (x, y) where x,y ∈ [0, 4] for a 5×5 matrix
    NeuronFiring { coordinates: Vec<(u8, u8), 25> },
    /// Request device capabilities JSON
    GetCapabilities,
}

/// Binary packet command IDs
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketCommand {
    NeuronFiring = 0x01,
    SetGpio = 0x02,
    SetPwm = 0x03,
    SetLedMatrix = 0x04,
    GetCapabilities = 0x05,
}

impl TryFrom<u8> for PacketCommand {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(PacketCommand::NeuronFiring),
            0x02 => Ok(PacketCommand::SetGpio),
            0x03 => Ok(PacketCommand::SetPwm),
            0x04 => Ok(PacketCommand::SetLedMatrix),
            0x05 => Ok(PacketCommand::GetCapabilities),
            _ => Err(()),
        }
    }
}

/// Transport-agnostic protocol handler
///
/// This handles protocol parsing and command buffering.
/// It does NOT handle transport connection/disconnection.
pub struct Protocol {
    device_name: &'static str,
    /// Receive buffer for incoming packets (from any transport)
    receive_buffer: Vec<u8, 256>,
    /// Connection status (managed by application, not protocol)
    connected: bool,
}

impl Protocol {
    /// Create a new protocol handler
    pub fn new(device_name: &'static str) -> Self {
        Self {
            device_name,
            receive_buffer: Vec::new(),
            connected: false,
        }
    }
    
    /// Get device name
    pub fn device_name(&self) -> &str {
        self.device_name
    }
    
    /// Check if connected (application-managed)
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Set connection status (called by application)
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        if !connected {
            // Clear buffer on disconnect
            self.receive_buffer.clear();
        }
    }
    
    /// Process incoming data from transport layer
    ///
    /// This appends data to the internal buffer for parsing.
    /// Call `receive_command()` to extract parsed commands.
    ///
    /// **Transport Independence:** This method accepts bytes from ANY source:
    /// - BLE notification data
    /// - USB CDC read buffer
    /// - UART RX buffer
    /// - WiFi socket data
    pub fn process_received_data(&mut self, data: &[u8]) {
        for &byte in data {
            if self.receive_buffer.push(byte).is_err() {
                // Buffer full - clear and restart
                // This handles malformed packets or overflow
                self.receive_buffer.clear();
                break;
            }
        }
    }
    
    /// Parse and consume the next command from the buffer
    ///
    /// Returns `Some(Command)` if a complete, valid command was parsed.
    /// Returns `None` if buffer is empty or packet is incomplete/invalid.
    pub fn receive_command(&mut self) -> Option<Command> {
        // Try neuron firing packet
        if let Some(coords) = self.parse_neuron_firing_packet() {
            return Some(Command::NeuronFiring { coordinates: coords });
        }
        
        // TODO: Add other packet types (GPIO, PWM, LED matrix, capabilities)
        
        None
    }
    
    /// Parse neuron firing packet
    ///
    /// Format: [0x01] [count] [x1, y1, x2, y2, ...]
    fn parse_neuron_firing_packet(&mut self) -> Option<Vec<(u8, u8), 25>> {
        if self.receive_buffer.len() < 2 {
            return None;
        }
        
        if self.receive_buffer[0] != PacketCommand::NeuronFiring as u8 {
            return None;
        }
        
        let count = self.receive_buffer[1] as usize;
        if count > 25 || self.receive_buffer.len() < 2 + count * 2 {
            return None;
        }
        
        let mut coords = Vec::new();
        for i in 0..count {
            let x = self.receive_buffer[2 + i * 2];
            let y = self.receive_buffer[2 + i * 2 + 1];
            if coords.push((x, y)).is_err() {
                break;
            }
        }
        
        // Consume processed bytes
        let consumed = 2 + count * 2;
        self.consume_bytes(consumed);
        
        Some(coords)
    }
    
    /// Remove consumed bytes from buffer
    fn consume_bytes(&mut self, count: usize) {
        if count >= self.receive_buffer.len() {
            self.receive_buffer.clear();
            return;
        }
        
        // Shift remaining data to front
        for i in count..self.receive_buffer.len() {
            self.receive_buffer[i - count] = self.receive_buffer[i];
        }
        
        // Truncate to new length
        for _ in 0..count {
            if self.receive_buffer.pop().is_none() {
                break;
            }
        }
    }
    
    /// Format capabilities JSON into byte buffer
    ///
    /// Example capabilities string:
    /// ```json
    /// {
    ///   "sensors": {"accel": true, "mag": true},
    ///   "gpio": {"digital": 8, "pwm": 8},
    ///   "display": {"matrix": true}
    /// }
    /// ```
    pub fn get_capabilities_data(&self, caps: &str) -> Vec<u8, 256> {
        let mut buffer = Vec::new();
        for &byte in caps.as_bytes() {
            if buffer.push(byte).is_err() {
                break;
            }
        }
        buffer
    }
}

// Re-export for backward compatibility
pub use Protocol as BluetoothProtocol;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_creation() {
        let protocol = Protocol::new("FEAGI-test");
        assert_eq!(protocol.device_name(), "FEAGI-test");
        assert!(!protocol.is_connected());
    }
    
    #[test]
    fn test_connection_status() {
        let mut protocol = Protocol::new("FEAGI-test");
        
        protocol.set_connected(true);
        assert!(protocol.is_connected());
        protocol.set_connected(false);
        assert!(!protocol.is_connected());
    }
    
    #[test]
    fn test_parse_neuron_firing_valid() {
        let mut protocol = Protocol::new("FEAGI-test");
        
        // Valid packet
        let packet = [0x01, 0x02, 0x01, 0x02, 0x03, 0x04];
        protocol.process_received_data(&packet);
        
        let result = protocol.receive_command();
        assert!(result.is_some());
        
        if let Some(Command::NeuronFiring { coordinates }) = result {
            assert_eq!(coordinates.len(), 2);
            assert_eq!(coordinates[0], (1, 2));
            assert_eq!(coordinates[1], (3, 4));
        } else {
            panic!("Expected NeuronFiring command");
        }
    }
    
    #[test]
    fn test_buffer_overflow_handling() {
        let mut protocol = Protocol::new("FEAGI-test");
        
        // Fill buffer beyond capacity
        let mut large_data = heapless::Vec::<u8, 300>::new();
        for i in 0..300 {
            let _ = large_data.push(i as u8);
        }
        protocol.process_received_data(&large_data);
        
        // Buffer should handle overflow
        let packet = [0x01, 0x01, 0x05, 0x06];
        protocol.process_received_data(&packet);
        let result = protocol.receive_command();
        assert!(result.is_some());
    }
    
    #[test]
    fn test_disconnect_clears_buffer() {
        let mut protocol = Protocol::new("FEAGI-test");
        
        protocol.process_received_data(&[0x01, 0x02, 0x03]);
        protocol.set_connected(false);
        
        // Buffer should be cleared
        let packet = [0x01, 0x01, 0x05, 0x06];
        protocol.process_received_data(&packet);
        
        let result = protocol.receive_command();
        assert!(result.is_some());
    }
}

