// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Session identifier (8 bytes) used for agent registration and routing.

/// Opaque 8-byte session identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SessionID(pub [u8; 8]);

impl SessionID {
    /// Returns the raw bytes.
    pub fn bytes(&self) -> &[u8; 8] {
        &self.0
    }
}
