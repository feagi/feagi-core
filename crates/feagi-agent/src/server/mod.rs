// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Server-side agent handler for FEAGI host (ZMQ/WS registration and streams).

mod auth;
#[cfg(feature = "server")]
mod feagi_agent_handler;

pub use auth::DummyAuth;
#[cfg(feature = "server")]
pub use feagi_agent_handler::FeagiAgentHandler;
