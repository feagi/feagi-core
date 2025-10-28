/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! # Neuron ID Management Module (Future)
//!
//! This module will provide:
//! - Dynamic ID allocation with Roaring Bitmaps
//! - ID recycling for deleted neurons
//! - Fast ID → model type routing
//! - Support for 1B+ neurons
//!
//! ## Current Status
//! - Phase 0: Structure established, not yet implemented
//! - Existing ID management in `feagi-plasticity/src/neuron_id_manager.rs`
//!
//! ## Future Migration
//! When implementing multi-model architecture:
//! 1. Migrate from feagi-plasticity to here
//! 2. Add Roaring Bitmap support
//! 3. Implement dynamic range allocation
//! 4. Add model-aware routing
//!
//! See: `feagi-core/docs/MULTI_MODEL_NEURON_ARCHITECTURE.md` Section 2

pub mod types;
pub mod id_manager;
pub mod router;

pub use types::NeuronArrayType;
// Future exports (currently commented out):
// pub use id_manager::NeuronIdManager;
// pub use router::NeuronRouter;


