// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Modulator instance endpoints. Driver areas are created and deleted with the instance.

use crate::common::{ApiError, ApiResult, ApiState, Json, Path, Query, State};
use feagi_evolutionary::ModulatorWrite;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ModulatorIdQuery {
    pub modulator_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteModulatorQuery {
    pub modulator_id: String,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Deserialize)]
pub struct RenameModulatorRequest {
    pub old_modulator_id: String,
    pub new_modulator_id: String,
}

/// List the fixed modulator types.
#[utoipa::path(get, path = "/v1/modulator/types", tag = "modulator")]
pub async fn get_modulator_types(State(state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    let types = state
        .genome_service
        .list_modulator_types()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(types))
}

/// List modulator instances in the loaded genome.
#[utoipa::path(get, path = "/v1/modulator/modulators", tag = "modulator")]
pub async fn get_modulators(State(state): State<ApiState>) -> ApiResult<Json<Value>> {
    let modulators = state
        .genome_service
        .list_modulators()
        .await
        .map_err(ApiError::from)?;
    Ok(Json(modulators))
}

/// Read one modulator instance.
#[utoipa::path(
    get,
    path = "/v1/modulator/modulator/{modulator_id}",
    tag = "modulator"
)]
pub async fn get_modulator(
    State(state): State<ApiState>,
    Path(modulator_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let instance = state
        .genome_service
        .get_modulator(&modulator_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(instance))
}

/// Create a modulator instance and its driver area.
#[utoipa::path(post, path = "/v1/modulator/modulator", tag = "modulator")]
pub async fn post_modulator(
    State(state): State<ApiState>,
    Query(query): Query<ModulatorIdQuery>,
    Json(write): Json<ModulatorWrite>,
) -> ApiResult<Json<Value>> {
    let instance = state
        .genome_service
        .create_modulator(query.modulator_id, write)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(instance))
}

/// Update a modulator instance and its driver locked fields.
#[utoipa::path(put, path = "/v1/modulator/modulator", tag = "modulator")]
pub async fn put_modulator(
    State(state): State<ApiState>,
    Query(query): Query<ModulatorIdQuery>,
    Json(write): Json<ModulatorWrite>,
) -> ApiResult<Json<Value>> {
    let instance = state
        .genome_service
        .update_modulator(&query.modulator_id, write)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(instance))
}

/// Rename a modulator and every subscriber reference.
#[utoipa::path(put, path = "/v1/modulator/rename", tag = "modulator")]
pub async fn put_rename_modulator(
    State(state): State<ApiState>,
    Json(request): Json<RenameModulatorRequest>,
) -> ApiResult<()> {
    state
        .genome_service
        .rename_modulator(&request.old_modulator_id, request.new_modulator_id)
        .await
        .map_err(ApiError::from)?;
    Ok(())
}

/// Areas and mappings that subscribe to a modulator.
#[utoipa::path(get, path = "/v1/modulator/usage/{modulator_id}", tag = "modulator")]
pub async fn get_modulator_usage(
    State(state): State<ApiState>,
    Path(modulator_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let usage = state
        .genome_service
        .modulator_usage(&modulator_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(usage))
}

/// Delete a modulator. `force` removes subscriptions first.
#[utoipa::path(delete, path = "/v1/modulator/modulator", tag = "modulator")]
pub async fn delete_modulator(
    State(state): State<ApiState>,
    Query(query): Query<DeleteModulatorQuery>,
) -> ApiResult<()> {
    state
        .genome_service
        .delete_modulator(&query.modulator_id, query.force)
        .await
        .map_err(ApiError::from)?;
    Ok(())
}
