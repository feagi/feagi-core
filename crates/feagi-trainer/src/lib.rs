//! # feagi-trainer
//!
//! Open-source (Apache-2.0) train / evaluate / benchmark **engine** for FEAGI spiking
//! neural brains. This crate is the engine/library and its public contracts only; it
//! contains no UI and makes no Composer/cloud calls. The closed-source "FEAGI Trainer" app
//! consumes it one-way (ADR-006).
//!
//! This initial vertical slice exposes the public [`contracts`] (v1): `DatasetManifest`,
//! `IRSample`, `RunSpec`, and `Scorecard`. Engine wiring (adapters, samplers, encoder/
//! decoder binding selectors over `feagi-sensorimotor`, metric packs, and deterministic run
//! execution) lands next.

pub mod contracts;
