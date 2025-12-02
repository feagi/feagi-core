// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::security::AuthContext;

/// Future permissions (stub)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Permission {
    // Neuron permissions
    NeuronRead,
    NeuronCreate,
    NeuronDelete,
    
    // Cortical area permissions
    CorticalAreaRead,
    CorticalAreaCreate,
    CorticalAreaUpdate,
    CorticalAreaDelete,
    
    // Brain region permissions
    BrainRegionRead,
    BrainRegionCreate,
    BrainRegionUpdate,
    BrainRegionDelete,
    
    // Genome permissions
    GenomeLoad,
    GenomeSave,
    GenomeValidate,
    
    // Analytics permissions
    AnalyticsRead,
    
    // System permissions
    SystemAdmin,
    SystemRead,
}

/// Future authorizer (stub)
pub struct Authorizer;

impl Authorizer {
    /// Authorize a permission (stub - always allows)
    pub fn authorize(_ctx: &AuthContext, _perm: Permission) -> Result<(), AuthzError> {
        Ok(())  // Stub: always allow
    }
}

/// Authorization error (stub)
#[derive(Debug, Clone)]
pub struct AuthzError {
    pub message: String,
}

impl AuthzError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AuthzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AuthzError {}





