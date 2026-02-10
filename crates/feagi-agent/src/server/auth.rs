// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! No-op auth for development; FEAGI host accepts all agents.

/// Dummy auth that accepts all registration requests.
#[derive(Debug, Clone)]
pub struct DummyAuth;
