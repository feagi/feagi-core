//! # feagi-trainer
//!
//! Open-source (Apache-2.0) train / evaluate / benchmark **engine** for FEAGI spiking
//! neural brains. This crate is the engine/library and its public contracts only; it
//! contains no UI and makes no Composer/cloud calls. The closed-source "FEAGI Trainer" app
//! consumes it one-way (ADR-006).
//!
//! This vertical slice exposes the public [`contracts`] (v1) and the pure data-pipeline
//! plugin axes — [`plugins`] interfaces with concrete [`adapters`], [`samplers`], and
//! [`metrics`] implementations (the IRIS tabular-classification path). The FEAGI binding
//! selectors and run execution (behind a runtime abstraction, remote/ZMQ first) land next.

pub mod adapters;
pub mod binding;
pub mod contracts;
pub mod error;
pub mod executor;
pub mod metrics;
pub mod plugins;
pub mod run_config;
pub mod samplers;
