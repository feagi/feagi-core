// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! FEAGI Bluetooth Protocol Layer (Platform-Agnostic)
//!
//! This module defines the binary protocol for FEAGI→Embodiment communication over BLE.
//! It is completely platform-agnostic and can be used with any BLE stack implementation
//! (TrouBLE, esp-idf, STM32 BLE, etc.).
//!
//! ## Protocol Design
//!
//! The protocol uses simple binary packets for efficiency in embedded environments:
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
//! | Command ID | Name | Payload | Description |
//! |------------|------|---------|-------------|
//! | 0x01 | NeuronFiring | count + (x,y) pairs | Fired neurons for LED matrix |
//! | 0x02 | SetGpio | pin + value | Digital GPIO control |
//! | 0x03 | SetPwm | pin + duty | PWM control |
//! | 0x04 | SetLedMatrix | 25 bytes | Full LED matrix update |
//! | 0x05 | GetCapabilities | none | Request device capabilities |
//!
//! ## Example: Neuron Firing
//!
//! To light LEDs at (1,2) and (3,4):
//!
//! ```text
//! [0x01] [0x02] [0x01, 0x02, 0x03, 0x04]
//!   │      │      └─ Coordinates: (1,2), (3,4)
//!   │      └─ Count: 2 neurons
//!   └─ Command: NeuronFiring
//! ```

#![no_std]

use heapless::Vec;

/// FEAGI Bluetooth commands
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
    /// Set a GPIO pin to high or low
    SetGpio { pin: u8, value: bool },
    /// Set PWM duty cycle (0-255) on a pin
    SetPwm { pin: u8, duty: u8 },
    /// Set full LED matrix (5x5 = 25 bytes)
    SetLedMatrix { data: [u8; 25] },
    /// Neuron firing coordinates for LED matrix visualization
    NeuronFiring { coordinates: Vec<(u8, u8), 25> },
    /// Request device capabilities JSON
    GetCapabilities,
}

/// BLE packet command types (binary protocol)
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

/// Bluetooth protocol service (platform-agnostic)
///
/// This handles protocol parsing and command buffering, but does NOT
/// handle the actual BLE communication (that's the platform's job).
pub struct BluetoothProtocol {
    device_name: &'static str,
    /// Receive buffer for incoming BLE packets
    receive_buffer: Vec<u8, 256>,
    /// Connection status (set by platform layer)
    connected: bool,
}

impl BluetoothProtocol {
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
    
    /// Check if BLE is connected (set by platform layer)
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Set connection status (called by platform BLE stack)
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
        if !connected {
            // Clear buffer on disconnect
            self.receive_buffer.clear();
        }
    }
    
    /// Process incoming BLE data from platform layer
    ///
    /// This appends data to the internal buffer for parsing.
    /// Call `receive_command()` to parse buffered data.
    pub fn process_received_data(&mut self, data: &[u8]) {
        // Append to receive buffer
        for &byte in data {
            if self.receive_buffer.push(byte).is_err() {
                // Buffer full - clear and start over
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
        // Try each packet type
        if let Some(coords) = self.parse_neuron_firing_packet() {
            return Some(Command::NeuronFiring { coordinates: coords });
        }
        
        // TODO: Add other packet types (GPIO, PWM, etc.)
        
        None
    }
    
    /// Parse neuron firing packet from buffer
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
                break; // Max 25 coordinates
            }
        }
        
        // Consume processed bytes from buffer
        let consumed = 2 + count * 2;
        self.consume_bytes(consumed);
        
        Some(coords)
    }
    
    /// Remove consumed bytes from the front of the buffer
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
    
    /// Format capabilities JSON into a byte buffer
    ///
    /// Example capabilities string:
    /// ```json
    /// {
    ///   "sensors": {"accel": true, "mag": true, "temp": true, "buttons": true},
    ///   "gpio": {"digital": 8, "analog": 3, "pwm": 8},
    ///   "display": {"matrix": true}
    /// }
    /// ```
    pub fn get_capabilities_data(&self, caps: &str) -> Vec<u8, 256> {
        let mut buffer = Vec::new();
        for &byte in caps.as_bytes() {
            if buffer.push(byte).is_err() {
                break; // Buffer full
            }
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_creation() {
        let protocol = BluetoothProtocol::new("FEAGI-test");
        assert_eq!(protocol.device_name(), "FEAGI-test");
        assert!(!protocol.is_connected());
    }
    
    #[test]
    fn test_connection_status() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        assert!(!protocol.is_connected());
        protocol.set_connected(true);
        assert!(protocol.is_connected());
        protocol.set_connected(false);
        assert!(!protocol.is_connected());
    }
    
    #[test]
    fn test_parse_neuron_firing_valid() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        // Valid packet: [0x01] [count=2] [x1=1, y1=2, x2=3, y2=4]
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
    fn test_parse_neuron_firing_invalid_header() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        // Invalid header
        let packet = [0x02, 0x01, 0x00, 0x00];
        protocol.process_received_data(&packet);
        
        let result = protocol.receive_command();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_parse_neuron_firing_incomplete() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        // Incomplete packet (missing data)
        let packet = [0x01, 0x02, 0x01]; // Missing y coordinate
        protocol.process_received_data(&packet);
        
        let result = protocol.receive_command();
        assert!(result.is_none());
    }
    
    #[test]
    fn test_parse_neuron_firing_max_coords() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        // Maximum 25 coordinates
        let mut packet = heapless::Vec::<u8, 256>::new();
        packet.push(0x01).unwrap(); // Command
        packet.push(25).unwrap();   // Count
        for i in 0..25 {
            packet.push(i as u8).unwrap();       // x
            packet.push((i + 1) as u8).unwrap(); // y
        }
        protocol.process_received_data(&packet);
        
        let result = protocol.receive_command();
        assert!(result.is_some());
        
        if let Some(Command::NeuronFiring { coordinates }) = result {
            assert_eq!(coordinates.len(), 25);
        } else {
            panic!("Expected NeuronFiring command");
        }
    }
    
    #[test]
    fn test_buffer_overflow_handling() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        // Fill buffer beyond capacity
        let mut large_data = heapless::Vec::<u8, 300>::new();
        for i in 0..300 {
            let _ = large_data.push(i as u8);
        }
        protocol.process_received_data(&large_data);
        
        // Buffer should handle overflow (clears and starts over)
        // Verify protocol still works after overflow
        let packet = [0x01, 0x01, 0x05, 0x06]; // Valid packet
        protocol.process_received_data(&packet);
        let result = protocol.receive_command();
        assert!(result.is_some()); // Should parse successfully
    }
    
    #[test]
    fn test_get_capabilities_data() {
        let protocol = BluetoothProtocol::new("FEAGI-test");
        let caps = "{\"sensors\":{\"accel\":true}}";
        let data = protocol.get_capabilities_data(caps);
        
        assert_eq!(data.len(), caps.len());
        assert_eq!(data.as_slice(), caps.as_bytes());
    }
    
    #[test]
    fn test_disconnect_clears_buffer() {
        let mut protocol = BluetoothProtocol::new("FEAGI-test");
        
        // Add some data
        protocol.process_received_data(&[0x01, 0x02, 0x03]);
        
        // Disconnect should clear buffer
        protocol.set_connected(false);
        
        // Add new valid packet
        let packet = [0x01, 0x01, 0x05, 0x06];
        protocol.process_received_data(&packet);
        
        // Should parse new packet successfully (not confused by old data)
        let result = protocol.receive_command();
        assert!(result.is_some());
    }
}

