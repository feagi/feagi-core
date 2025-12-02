// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome-related service utilities for cortical area updates.

This module contains the change classification and parameter update logic
for efficient cortical area modifications without full brain rebuilds.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

pub mod change_classifier;
pub mod parameter_updater;

pub use change_classifier::{ChangeType, CorticalChangeClassifier};
pub use parameter_updater::CorticalParameterUpdater;

