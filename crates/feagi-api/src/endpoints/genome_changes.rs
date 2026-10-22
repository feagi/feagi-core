// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Genome change ledger endpoints (`/v1/genome/changes*`).
//!
//! Read the ordered changes made to this instance's genome, and replay a change
//! recorded on another instance. Callers identify themselves with the
//! `X-FEAGI-Agent-Id` header (see `middleware::change_attribution`).

use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use feagi_services::change_ledger::{
    apply_change, ApplyGenomeChangeOutcome, ApplyGenomeChangeRequest, ChangeLedger, ChangesPage,
};
use serde::Deserialize;
use std::sync::Arc;

/// Request header carrying the caller's agent id for change attribution.
pub const AGENT_ID_HEADER: &str = "x-feagi-agent-id";

/// Query for `GET /v1/genome/changes`.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ChangesQuery {
    /// Return changes with a sequence greater than this cursor (0 for all held changes).
    pub since: u64,
    /// Maximum changes to return; at most `[genome].change_ledger_session_capacity`.
    /// Omitted means the configured capacity.
    pub limit: Option<usize>,
}

fn ledger(state: &ApiState) -> ApiResult<&Arc<ChangeLedger>> {
    state
        .change_ledger
        .as_ref()
        .ok_or_else(|| ApiError::not_implemented("Genome change ledger is not enabled"))
}

/// Changes recorded after `since`, oldest first.
///
/// `gap: true` means changes after the cursor were evicted from memory; the caller
/// must resynchronize from the full genome before continuing.
#[utoipa::path(
    get,
    path = "/v1/genome/changes",
    params(ChangesQuery),
    responses(
        (status = 200, description = "Ordered genome changes", body = serde_json::Value),
        (status = 400, description = "Invalid cursor or limit"),
        (status = 501, description = "Change ledger not enabled")
    ),
    tag = "genome"
)]
pub async fn get_changes(
    State(state): State<ApiState>,
    Query(query): Query<ChangesQuery>,
) -> ApiResult<Json<ChangesPage>> {
    let ledger = ledger(&state)?;
    let limit = query.limit.unwrap_or(ledger.config().session_capacity);
    Ok(Json(ledger.changes_since(query.since, limit)?))
}

/// Replay a change recorded on another instance.
///
/// The change keeps its original id, group, timestamp and agent, and is recorded
/// here with `origin: replayed`. Submitting a change id that is already present is
/// a no-op (`already_applied`). A change whose target no longer exists fails with
/// the service error (for example 404) and is not recorded.
#[utoipa::path(
    post,
    path = "/v1/genome/changes/apply",
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Applied or already present", body = serde_json::Value),
        (status = 400, description = "Invalid or non-replayable change"),
        (status = 404, description = "Change target not found"),
        (status = 409, description = "Change conflicts with current state"),
        (status = 501, description = "Change ledger not enabled")
    ),
    tag = "genome"
)]
pub async fn post_apply_change(
    State(state): State<ApiState>,
    Json(request): Json<ApplyGenomeChangeRequest>,
) -> ApiResult<Json<ApplyGenomeChangeOutcome>> {
    let ledger = ledger(&state)?;
    let outcome = apply_change(
        request,
        ledger,
        state.genome_service.as_ref(),
        state.connectome_service.as_ref(),
    )
    .await?;
    Ok(Json(outcome))
}
