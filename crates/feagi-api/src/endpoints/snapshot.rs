// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
 * FEAGI v1 Snapshot API
 * 
 * Endpoints for creating, managing, and restoring brain snapshots
 * Maps to Python: feagi/api/v1/snapshot.py
 */

use crate::common::{ApiError, ApiResult, State, Json};
use crate::transports::http::server::ApiState;
