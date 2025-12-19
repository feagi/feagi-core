// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Event types for signal-based communication between burst engine and PNS
//!
//! This module defines events that flow through FeagiSignal for decoupling:
//! - Burst Engine → PNS: Visualization data, motor commands
//! - PNS → Burst Engine: Sensory data, agent lifecycle events

use super::types::SharedFBC;

/// Event emitted when visualization data is ready to be published
///
/// # Flow
/// 1. Burst engine computes neural activity
/// 2. Burst engine wraps data in FBC and emits this event
/// 3. PNS visualization stream subscribes and publishes via ZMQ/UDP
///
/// # Example
/// ```no_run
/// use feagi_io::core::events::VisualizationReadyEvent;
/// use feagi_data_serialization::FeagiByteContainer;
/// use feagi_data_structures::FeagiSignal;
/// use std::sync::{Arc, Mutex};
///
/// // Burst engine side
/// let mut viz_signal = FeagiSignal::<VisualizationReadyEvent>::new();
///
/// // PNS subscribes
/// viz_signal.connect(|event| {
///     // Publish via transport
///     println!("Publishing {} bytes", event.fbc.get_byte_ref().len());
/// });
///
/// // Burst engine emits
/// let fbc = Arc::new(FeagiByteContainer::new_empty());
/// viz_signal.emit(&VisualizationReadyEvent { fbc });
/// ```
#[derive(Debug, Clone)]
pub struct VisualizationReadyEvent {
    /// Shared reference to FBC containing neural activity data
    pub fbc: SharedFBC,
}

/// Event emitted when motor commands are ready to be sent to an agent
///
/// # Flow
/// 1. Burst engine computes motor outputs
/// 2. Burst engine wraps commands in FBC and emits this event
/// 3. PNS motor stream subscribes and sends to specific agent
#[derive(Debug, Clone)]
pub struct MotorCommandEvent {
    /// Target agent identifier
    pub agent_id: String,
    /// Shared reference to FBC containing motor command data
    pub fbc: SharedFBC,
}

/// Event emitted when sensory data is received from an agent
///
/// # Flow
/// 1. PNS receives sensory data via ZMQ/UDP
/// 2. PNS decompresses and wraps in FBC
/// 3. PNS emits this event
/// 4. Burst engine subscribes and injects into NPU
///
/// # Example
/// ```no_run
/// use feagi_io::core::events::SensoryDataEvent;
/// use feagi_data_serialization::FeagiByteContainer;
/// use feagi_data_structures::FeagiSignal;
/// use std::sync::{Arc, Mutex};
///
/// // PNS side
/// let mut sensory_signal = FeagiSignal::<SensoryDataEvent>::new();
///
/// // Burst engine subscribes
/// sensory_signal.connect(|event| {
///     // Inject into NPU
///     println!("Received sensory data from {}", event.agent_id);
/// });
///
/// // PNS emits when data received
/// let fbc = Arc::new(FeagiByteContainer::new_empty());
/// sensory_signal.emit(&SensoryDataEvent {
///     agent_id: "agent-001".to_string(),
///     fbc,
/// });
/// ```
#[derive(Debug, Clone)]
pub struct SensoryDataEvent {
    /// Source agent identifier
    pub agent_id: String,
    /// Shared reference to FBC containing sensory data
    pub fbc: SharedFBC,
}

/// Event emitted when a new agent registers
///
/// # Flow
/// 1. PNS receives registration request
/// 2. PNS validates and registers agent
/// 3. PNS emits this event
/// 4. Burst engine subscribes to set up agent-specific resources
#[derive(Debug, Clone)]
pub struct AgentRegisteredEvent {
    /// Registered agent identifier
    pub agent_id: String,
    /// Agent type (e.g., "sensory", "motor", "visualization")
    pub agent_type: String,
    /// Agent capabilities (JSON for flexibility)
    pub capabilities: String,
}

/// Event emitted when an agent disconnects or is deregistered
///
/// # Flow
/// 1. PNS detects agent timeout or receives deregistration request
/// 2. PNS cleans up agent resources
/// 3. PNS emits this event
/// 4. Burst engine subscribes to clean up agent-specific state
#[derive(Debug, Clone)]
pub struct AgentDisconnectedEvent {
    /// Disconnected agent identifier
    pub agent_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_data_serialization::FeagiByteContainer;
    use feagi_data_structures::FeagiSignal;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_visualization_event_signal() {
        let mut signal = FeagiSignal::<VisualizationReadyEvent>::new();
        let received = Arc::new(Mutex::new(false));
        let received_clone = Arc::clone(&received);

        signal.connect(move |_event| {
            *received_clone.lock().unwrap() = true;
        });

        let fbc = Arc::new(FeagiByteContainer::new_empty());
        signal.emit(&VisualizationReadyEvent { fbc });

        assert!(*received.lock().unwrap());
    }

    #[test]
    fn test_sensory_event_signal() {
        let mut signal = FeagiSignal::<SensoryDataEvent>::new();
        let agent_id_received = Arc::new(Mutex::new(String::new()));
        let agent_id_clone = Arc::clone(&agent_id_received);

        signal.connect(move |event| {
            *agent_id_clone.lock().unwrap() = event.agent_id.clone();
        });

        let fbc = Arc::new(FeagiByteContainer::new_empty());
        signal.emit(&SensoryDataEvent {
            agent_id: "test-agent".to_string(),
            fbc,
        });

        assert_eq!(*agent_id_received.lock().unwrap(), "test-agent");
    }

    #[test]
    fn test_agent_registered_event() {
        let mut signal = FeagiSignal::<AgentRegisteredEvent>::new();
        let count = Arc::new(Mutex::new(0));
        let count_clone = Arc::clone(&count);

        signal.connect(move |_event| {
            *count_clone.lock().unwrap() += 1;
        });

        signal.emit(&AgentRegisteredEvent {
            agent_id: "agent-001".to_string(),
            agent_type: "sensory".to_string(),
            capabilities: "{}".to_string(),
        });

        assert_eq!(*count.lock().unwrap(), 1);
    }
}
