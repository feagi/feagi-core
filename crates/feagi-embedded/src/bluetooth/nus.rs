// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Nordic UART Service (NUS) and FEAGI Service UUID Definitions
//!
//! This module defines the BLE service and characteristic UUIDs used for
//! FEAGI communication. We use both the standard Nordic UART Service (NUS)
//! for simple bidirectional communication and custom FEAGI service UUIDs
//! for structured data.
//!
//! ## Nordic UART Service (NUS)
//!
//! NUS is a simple, widely-supported BLE service that mimics a UART interface:
//!
//! - **Service UUID**: `6E400001-B5A3-F393-E0A9-E50E24DCCA9E`
//! - **TX Characteristic**: `6E400003-B5A3-F393-E0A9-E50E24DCCA9E` (Notify)
//!   - Device → Client (micro:bit sends sensor data)
//! - **RX Characteristic**: `6E400002-B5A3-F393-E0A9-E50E24DCCA9E` (Write)
//!   - Client → Device (FEAGI sends commands)
//!
//! ## FEAGI Custom Service (Alternative)
//!
//! For more structured communication, FEAGI defines custom characteristics:
//!
//! - **Service UUID**: `E95D0753-251D-470A-A062-FA1922DFA9A8`
//! - **Sensor Data** (Notify): `E95D0754-251D-470A-A062-FA1922DFA9A8`
//! - **Neuron Data** (Write): `E95D0755-251D-470A-A062-FA1922DFA9A8`
//! - **GPIO Control** (Write): `E95D0756-251D-470A-A062-FA1922DFA9A8`
//! - **LED Matrix** (Write): `E95D0757-251D-470A-A062-FA1922DFA9A8`
//! - **Capabilities** (Read): `E95D0758-251D-470A-A062-FA1922DFA9A8`
//!
//! ## Usage
//!
//! ```rust
//! use feagi_embedded::bluetooth::nus::*;
//!
//! // Use NUS for simple communication
//! let service_uuid = NUS_SERVICE_UUID;
//! let rx_uuid = NUS_RX_CHAR_UUID; // Client writes here
//! let tx_uuid = NUS_TX_CHAR_UUID; // Client reads/subscribes here
//!
//! // Or use FEAGI service for structured data
//! let service_uuid = FEAGI_SERVICE_UUID;
//! let neuron_uuid = NEURON_DATA_CHAR_UUID;
//! ```

#![no_std]

// ============================================================================
// Nordic UART Service (NUS) - Standard BLE service for serial-like communication
// ============================================================================

/// Nordic UART Service UUID: 6E400001-B5A3-F393-E0A9-E50E24DCCA9E
///
/// This is the primary service UUID for NUS.
pub const NUS_SERVICE_UUID: [u8; 16] = [
    0x6e, 0x40, 0x00, 0x01, 0xb5, 0xa3, 0xf3, 0x93,
    0xe0, 0xa9, 0xe5, 0x0e, 0x24, 0xdc, 0xca, 0x9e,
];

/// NUS TX Characteristic UUID: 6E400003-B5A3-F393-E0A9-E50E24DCCA9E
///
/// **Direction**: Device → Client (Notify)
/// **Purpose**: Device sends sensor data or status updates to client
pub const NUS_TX_CHAR_UUID: [u8; 16] = [
    0x6e, 0x40, 0x00, 0x03, 0xb5, 0xa3, 0xf3, 0x93,
    0xe0, 0xa9, 0xe5, 0x0e, 0x24, 0xdc, 0xca, 0x9e,
];

/// NUS RX Characteristic UUID: 6E400002-B5A3-F393-E0A9-E50E24DCCA9E
///
/// **Direction**: Client → Device (Write)
/// **Purpose**: Client sends commands or motor data to device
pub const NUS_RX_CHAR_UUID: [u8; 16] = [
    0x6e, 0x40, 0x00, 0x02, 0xb5, 0xa3, 0xf3, 0x93,
    0xe0, 0xa9, 0xe5, 0x0e, 0x24, 0xdc, 0xca, 0x9e,
];

// ============================================================================
// FEAGI Custom Service - Structured characteristics for different data types
// ============================================================================

/// FEAGI BLE Service UUID: E95D0753-251D-470A-A062-FA1922DFA9A8
pub const FEAGI_SERVICE_UUID: [u8; 16] = [
    0xe9, 0x5d, 0x07, 0x53, 0x25, 0x1d, 0x47, 0x0a,
    0xa0, 0x62, 0xfa, 0x19, 0x22, 0xdf, 0xa9, 0xa8,
];

/// Sensor Data Characteristic UUID: E95D0754-251D-470A-A062-FA1922DFA9A8
///
/// **Direction**: Device → Client (Notify)
/// **Purpose**: Periodic sensor readings (accel, mag, temp, buttons)
/// **Format**: JSON string
pub const SENSOR_DATA_CHAR_UUID: [u8; 16] = [
    0xe9, 0x5d, 0x07, 0x54, 0x25, 0x1d, 0x47, 0x0a,
    0xa0, 0x62, 0xfa, 0x19, 0x22, 0xdf, 0xa9, 0xa8,
];

/// Neuron Data Characteristic UUID: E95D0755-251D-470A-A062-FA1922DFA9A8
///
/// **Direction**: Client → Device (Write)
/// **Purpose**: Neuron firing coordinates for LED matrix visualization
/// **Format**: Binary packet (see protocol.rs)
pub const NEURON_DATA_CHAR_UUID: [u8; 16] = [
    0xe9, 0x5d, 0x07, 0x55, 0x25, 0x1d, 0x47, 0x0a,
    0xa0, 0x62, 0xfa, 0x19, 0x22, 0xdf, 0xa9, 0xa8,
];

/// GPIO Control Characteristic UUID: E95D0756-251D-470A-A062-FA1922DFA9A8
///
/// **Direction**: Client → Device (Write)
/// **Purpose**: Digital I/O and PWM control
/// **Format**: Binary packet (see protocol.rs)
pub const GPIO_CONTROL_CHAR_UUID: [u8; 16] = [
    0xe9, 0x5d, 0x07, 0x56, 0x25, 0x1d, 0x47, 0x0a,
    0xa0, 0x62, 0xfa, 0x19, 0x22, 0xdf, 0xa9, 0xa8,
];

/// LED Matrix Characteristic UUID: E95D0757-251D-470A-A062-FA1922DFA9A8
///
/// **Direction**: Client → Device (Write)
/// **Purpose**: Full 5×5 LED matrix update
/// **Format**: 25 bytes (brightness values 0-255)
pub const LED_MATRIX_CHAR_UUID: [u8; 16] = [
    0xe9, 0x5d, 0x07, 0x57, 0x25, 0x1d, 0x47, 0x0a,
    0xa0, 0x62, 0xfa, 0x19, 0x22, 0xdf, 0xa9, 0xa8,
];

/// Capabilities Characteristic UUID: E95D0758-251D-470A-A062-FA1922DFA9A8
///
/// **Direction**: Device → Client (Read)
/// **Purpose**: Device capabilities JSON (sensors, GPIO, display)
/// **Format**: JSON string
pub const CAPABILITIES_CHAR_UUID: [u8; 16] = [
    0xe9, 0x5d, 0x07, 0x58, 0x25, 0x1d, 0x47, 0x0a,
    0xa0, 0x62, 0xfa, 0x19, 0x22, 0xdf, 0xa9, 0xa8,
];

// ============================================================================
// Default Device Names
// ============================================================================

/// Default device name for FEAGI-enabled micro:bit
pub const FEAGI_MICROBIT_NAME: &str = "FEAGI-microbit";

/// Default device name for FEAGI-enabled ESP32
pub const FEAGI_ESP32_NAME: &str = "FEAGI-esp32";

/// Default device name for generic FEAGI robot
pub const FEAGI_ROBOT_NAME: &str = "FEAGI-robot";

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert UUID byte array to standard UUID string format
///
/// Example: `[0x6e, 0x40, ...]` → `"6e400001-b5a3-f393-e0a9-e50e24dcca9e"`
#[cfg(feature = "alloc")]
pub fn uuid_to_string(uuid: &[u8; 16]) -> alloc::string::String {
    use alloc::format;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        uuid[0], uuid[1], uuid[2], uuid[3],
        uuid[4], uuid[5],
        uuid[6], uuid[7],
        uuid[8], uuid[9],
        uuid[10], uuid[11], uuid[12], uuid[13], uuid[14], uuid[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nus_uuids_are_unique() {
        assert_ne!(NUS_SERVICE_UUID, NUS_TX_CHAR_UUID);
        assert_ne!(NUS_SERVICE_UUID, NUS_RX_CHAR_UUID);
        assert_ne!(NUS_TX_CHAR_UUID, NUS_RX_CHAR_UUID);
    }
    
    #[test]
    fn test_feagi_uuids_are_unique() {
        let uuids = [
            FEAGI_SERVICE_UUID,
            SENSOR_DATA_CHAR_UUID,
            NEURON_DATA_CHAR_UUID,
            GPIO_CONTROL_CHAR_UUID,
            LED_MATRIX_CHAR_UUID,
            CAPABILITIES_CHAR_UUID,
        ];
        
        // Check all pairs are different
        for i in 0..uuids.len() {
            for j in (i+1)..uuids.len() {
                assert_ne!(uuids[i], uuids[j], "UUIDs at {} and {} are identical", i, j);
            }
        }
    }
    
    #[test]
    fn test_uuid_byte_order() {
        // NUS service UUID should start with 6E40
        assert_eq!(NUS_SERVICE_UUID[0], 0x6e);
        assert_eq!(NUS_SERVICE_UUID[1], 0x40);
        
        // FEAGI service UUID should start with E95D
        assert_eq!(FEAGI_SERVICE_UUID[0], 0xe9);
        assert_eq!(FEAGI_SERVICE_UUID[1], 0x5d);
    }
}

