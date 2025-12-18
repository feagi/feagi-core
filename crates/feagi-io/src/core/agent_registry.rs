// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Agent Registry - tracks all registered agents and their state
//
// This is the single source of truth for agent registration in FEAGI 2.0.
// It replaces the deprecated feagi-agent-registry crate.
//
// NOTE: AgentRegistry implementation moved to feagi-services to break circular dependency.
// This file now only re-exports for backward compatibility.

// Re-export types from feagi-services to break circular dependency
pub use feagi_services::types::agent_registry::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentTransport, AgentType, MotorCapability,
    SensoryCapability, VisionCapability, VisualizationCapability,
};
