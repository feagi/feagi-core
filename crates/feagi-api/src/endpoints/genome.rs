// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Genome API Endpoints - Exact port from Python `/v1/genome/*`

// Removed - using crate::common::State instead
use crate::amalgamation;
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use feagi_evolutionary::{
    decode_genome_artifact, encode_genome_artifact,
    validate_genome_artifact_file_name as validate_artifact_file_name, GENOME_ARTIFACT_MEDIA_TYPE,
};
use feagi_services::types::{GenomeInfo, LoadGenomeParams};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[cfg(feature = "http")]
use axum::extract::Multipart;
use axum::http::{header, HeaderMap, HeaderValue};

fn validate_genome_artifact_file_name(file_name: Option<&str>) -> ApiResult<()> {
    let file_name =
        file_name.ok_or_else(|| ApiError::invalid_input("Genome upload requires a file name"))?;
    validate_artifact_file_name(file_name)
        .map_err(|error| ApiError::invalid_input(error.to_string()))
}

fn decode_genome_artifact_for_json_service(artifact: &[u8]) -> ApiResult<String> {
    let genome = decode_genome_artifact(artifact)
        .map_err(|error| ApiError::invalid_input(error.to_string()))?;
    serde_json::to_string(&genome)
        .map_err(|error| ApiError::internal(format!("Failed to serialize genome: {error}")))
}

fn genome_artifact_download_headers(file_name: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(GENOME_ARTIFACT_MEDIA_TYPE),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static(file_name),
    );
    headers
}

/// Multipart file upload schema for Swagger UI.
///
/// This enables Swagger to show a file picker for `.genome` artifacts.
#[derive(Debug, Clone, utoipa::ToSchema)]
pub struct GenomeFileUploadForm {
    /// `.genome` artifact containing versioned genome JSON.
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

/// The host data a plan's wiring is validated against.
pub(crate) struct PlanHost {
    /// Size of each id the plan names that exists on the host (mapping ends and adapter ids).
    pub(crate) areas: HashMap<String, [u32; 3]>,
    pub(crate) morphologies: std::collections::HashSet<String>,
}

pub(crate) async fn read_plan_host(
    state: &ApiState,
    plan: &feagi_evolutionary::FitPlan,
    fitted_guest_areas: &HashMap<
        feagi_structures::genomic::cortical_area::CorticalID,
        feagi_structures::genomic::cortical_area::CorticalArea,
    >,
) -> Result<PlanHost, ApiError> {
    let connectome = state.connectome_service.as_ref();
    let mut areas: HashMap<String, [u32; 3]> = HashMap::new();
    for id in feagi_evolutionary::plan_host_lookup_ids(fitted_guest_areas, plan) {
        match connectome.get_cortical_area(&id).await {
            Ok(info) => {
                let (x, y, z) = info.dimensions;
                areas.insert(id, [x as u32, y as u32, z as u32]);
            }
            Err(e) if is_not_found(&e) => {}
            Err(e) => {
                return Err(ApiError::internal(format!(
                    "Failed to read host area {}: {}",
                    id, e
                )))
            }
        }
    }
    let morphologies = connectome
        .get_morphologies()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list host morphologies: {}", e)))?
        .into_keys()
        .collect();
    Ok(PlanHost {
        areas,
        morphologies,
    })
}

/// Validate the plan's wiring against `host` (pure; see `feagi_evolutionary::validate_plan`) and
/// check that every morphology its rules name, but the plan does not define, exists on the host
/// or in the guest genome.
pub(crate) fn check_plan_wiring(
    plan: &feagi_evolutionary::FitPlan,
    fitted_guest_areas: &HashMap<
        feagi_structures::genomic::cortical_area::CorticalID,
        feagi_structures::genomic::cortical_area::CorticalArea,
    >,
    guest_morphologies: &std::collections::HashSet<String>,
    host: &PlanHost,
) -> Result<(), ApiError> {
    feagi_evolutionary::validate_plan(
        &feagi_evolutionary::PlanContext {
            guest_areas: fitted_guest_areas,
            guest_morphologies,
            host_areas: &host.areas,
            host_morphologies: &host.morphologies,
        },
        plan,
    )
    .map_err(|e| ApiError::invalid_input(format!("Invalid fit_plan: {}", e)))?;
    let defined: std::collections::HashSet<&str> =
        plan.morphologies.iter().map(|m| m.name.as_str()).collect();
    for mapping in &plan.mappings {
        for rule in &mapping.rules {
            let name = rule
                .get("morphology_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if !defined.contains(name)
                && !host.morphologies.contains(name)
                && !guest_morphologies.contains(name)
            {
                return Err(ApiError::invalid_input(format!(
                    "Invalid fit_plan: mapping {} -> {} uses morphology '{}', which is not in the \
                     plan, the guest genome or the host",
                    mapping.src, mapping.dst, name
                )));
            }
        }
    }
    Ok(())
}

async fn queue_amalgamation_from_genome_json_str(
    state: &ApiState,
    genome_json: String,
    fit_plan: Option<feagi_evolutionary::FitPlan>,
    source_release_id: Option<String>,
    source_catalog: Option<String>,
) -> Result<String, ApiError> {
    // Only one pending amalgamation is supported per FEAGI session (matches BV workflow).
    {
        let lock = state.amalgamation_state.read();
        if lock.pending.is_some() {
            return Err(ApiError::invalid_input(
                "Amalgamation already pending; cancel it first via /v1/genome/amalgamation_cancellation",
            ));
        }
    }

    let mut genome = feagi_evolutionary::load_genome_from_json(&genome_json)
        .map_err(|e| ApiError::invalid_input(format!("Invalid genome payload: {}", e)))?;

    // Validate the fit plan now so a bad plan is refused at queue time with nothing pending, and
    // size the placement preview from the fitted areas (the footprint that will be created).
    // The host ends the wiring names are read now and kept: confirmation refuses when their
    // size changed in between.
    let mut fit_host_ends: HashMap<String, [u32; 3]> = HashMap::new();
    if let Some(plan) = &fit_plan {
        genome.cortical_areas = feagi_evolutionary::fit_areas(&genome.cortical_areas, plan)
            .map_err(|e| ApiError::invalid_input(format!("Invalid fit_plan: {}", e)))?;
        if plan.has_wiring() {
            let guest_morphologies: std::collections::HashSet<String> =
                genome.morphologies.morphology_ids().into_iter().collect();
            let host = read_plan_host(state, plan, &genome.cortical_areas).await?;
            check_plan_wiring(plan, &genome.cortical_areas, &guest_morphologies, &host)?;
            fit_host_ends = host.areas;
        }
    }

    let circuit_size = amalgamation::compute_circuit_size_from_runtime_genome(&genome);

    let amalgamation_id = Uuid::new_v4().to_string();
    let genome_title = genome.metadata.genome_title.clone();

    let summary = amalgamation::AmalgamationPendingSummary {
        amalgamation_id: amalgamation_id.clone(),
        genome_title,
        circuit_size,
    };

    let pending = amalgamation::AmalgamationPending {
        summary: summary.clone(),
        source: amalgamation::AmalgamationSource::ExternalGenome,
        genome_json,
        fit_plan,
        fit_host_ends,
        source_release_id,
        source_catalog,
    };

    {
        let mut lock = state.amalgamation_state.write();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        lock.history.push(amalgamation::AmalgamationHistoryEntry {
            amalgamation_id: summary.amalgamation_id.clone(),
            genome_title: summary.genome_title.clone(),
            circuit_size: summary.circuit_size,
            status: "pending".to_string(),
            timestamp_ms: now_ms,
        });
        lock.pending = Some(pending);
    }

    tracing::info!(
        target: "feagi-api",
        "🧬 [AMALGAMATION] Queued pending amalgamation id={} title='{}' circuit_size={:?}",
        summary.amalgamation_id,
        summary.genome_title,
        summary.circuit_size
    );

    Ok(amalgamation_id)
}

struct GenomeTransitionFlagGuard {
    in_progress: Arc<std::sync::atomic::AtomicBool>,
}

impl Drop for GenomeTransitionFlagGuard {
    fn drop(&mut self) {
        self.in_progress.store(false, Ordering::SeqCst);
    }
}

/// Mirrors prioritized genome transition in [feagi_state_manager::GenomeState] for health_check.
/// - [Self::enter][]: Loading
/// - [Self::succeed][]: Loaded (call when transition fully finished, including post-load agent IO)
/// - Drop without succeed: Error (failed or aborted transition)
struct GenomeTransitionStateLifecycle {
    /// The state before the transition began, put back when the transition was rolled back.
    #[cfg(feature = "services")]
    previous: feagi_state_manager::GenomeState,
}

impl GenomeTransitionStateLifecycle {
    fn enter() -> Self {
        #[cfg(feature = "services")]
        {
            let state = feagi_state_manager::StateManager::instance();
            let state = state.read();
            let previous = state.get_genome_state();
            state.set_genome_state(feagi_state_manager::GenomeState::Loading);
            Self { previous }
        }
        #[cfg(not(feature = "services"))]
        Self {}
    }

    /// The transition failed but the previous brain is live again: its state stands.
    #[cfg(feature = "services")]
    fn restore_previous(self) {
        feagi_state_manager::StateManager::instance()
            .read()
            .set_genome_state(self.previous);
        std::mem::forget(self);
    }

    fn succeed(self) {
        #[cfg(feature = "services")]
        {
            feagi_state_manager::StateManager::instance()
                .read()
                .set_genome_state(feagi_state_manager::GenomeState::Loaded);
        }
        std::mem::forget(self);
    }
}

impl Drop for GenomeTransitionStateLifecycle {
    fn drop(&mut self) {
        #[cfg(feature = "services")]
        {
            feagi_state_manager::StateManager::instance()
                .read()
                .set_genome_state(feagi_state_manager::GenomeState::Error);
        }
    }
}

// Production imports use `with_rollback_aware_genome_state_lifecycle`; this generic form
// stays for the lifecycle tests.
#[cfg(test)]
/// Runs a brain transition that is not a genome upload (a connectome import) under the
/// same health_check genome_state lifecycle, so a brain restored from a connectome reports
/// `loaded` like one loaded from a genome. Clients gate autosave on that state.
pub(crate) async fn with_genome_state_lifecycle<T, E>(
    transition: impl std::future::Future<Output = Result<T, E>>,
) -> Result<T, E> {
    let lifecycle = GenomeTransitionStateLifecycle::enter();
    let result = transition.await;
    if result.is_ok() {
        lifecycle.succeed();
    }
    result
}

/// Like [`with_genome_state_lifecycle`] for a service call that can roll back. A
/// `ServiceError::RolledBack` leaves the previous brain live, so the genome state returns to
/// what it was before the call (not `Error`) and the original error is returned.
#[cfg(feature = "services")]
pub(crate) async fn with_rollback_aware_genome_state_lifecycle<T>(
    transition: impl std::future::Future<Output = Result<T, feagi_services::ServiceError>>,
) -> Result<T, feagi_services::ServiceError> {
    let lifecycle = GenomeTransitionStateLifecycle::enter();
    match transition.await {
        Ok(value) => {
            lifecycle.succeed();
            Ok(value)
        }
        Err(feagi_services::ServiceError::RolledBack(original)) => {
            lifecycle.restore_previous();
            Err(*original)
        }
        // Dropping the lifecycle records Error.
        Err(error) => Err(error),
    }
}

/// Execute a genome load with strict priority over concurrent operations.
///
/// Guarantees:
/// - Only one genome transition may run at a time.
/// - Runtime is quiesced before load starts.
/// - Runtime frequency is updated from genome physiology.
/// - Runtime is restored to running state if it was running before transition.
async fn load_genome_with_priority(
    state: &ApiState,
    params: LoadGenomeParams,
    source: &str,
) -> ApiResult<GenomeInfo> {
    let _transition_lock = state.genome_transition_lock.try_write().map_err(|_| {
        ApiError::conflict(
            "Another genome transition is already in progress; wait for it to finish",
        )
    })?;
    state
        .genome_transition_in_progress
        .store(true, Ordering::SeqCst);
    let _guard = GenomeTransitionFlagGuard {
        in_progress: Arc::clone(&state.genome_transition_in_progress),
    };
    let genome_sm_lifecycle = GenomeTransitionStateLifecycle::enter();

    tracing::info!(
        target: "feagi-api",
        "🛑 Entering prioritized genome transition from {}",
        source
    );

    let runtime_service = state.runtime_service.as_ref();
    #[cfg(feature = "feagi-agent")]
    if let Some(handler) = &state.agent_handler {
        let deregistered_ids = {
            let mut guard = handler.lock().unwrap();
            guard.force_deregister_all_agents("forced by genome transition")
        };
        for agent_id in &deregistered_ids {
            runtime_service.unregister_motor_subscriptions(agent_id);
            runtime_service.unregister_visualization_subscriptions(agent_id);
        }
        tracing::info!(
            target: "feagi-api",
            "🔌 Forced deregistration for {} agents before genome transition",
            deregistered_ids.len()
        );
    }
    // Strict transition barrier: guarantee no stale subscriptions survive.
    runtime_service.clear_all_motor_subscriptions();
    runtime_service.clear_all_visualization_subscriptions();

    let runtime_status = runtime_service
        .get_status()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get runtime status: {}", e)))?;
    let runtime_was_running = runtime_status.is_running;

    if runtime_was_running {
        tracing::info!(
            target: "feagi-api",
            "Stopping burst engine before prioritized genome transition"
        );
        runtime_service.stop().await.map_err(|e| {
            ApiError::internal(format!(
                "Failed to stop burst engine before genome transition: {}",
                e
            ))
        })?;
    }

    let genome_service = state.genome_service.as_ref();
    let load_result = genome_service.load_genome(params).await;
    let genome_info = match load_result {
        Ok(info) => info,
        Err(e) => {
            if runtime_was_running {
                if let Err(restart_err) = runtime_service.start().await {
                    tracing::warn!(
                        target: "feagi-api",
                        "Failed to restore runtime after failed genome load (source={}): {}",
                        source,
                        restart_err
                    );
                }
            }
            return Err(ApiError::internal(format!("Failed to load genome: {}", e)));
        }
    };

    let burst_frequency_hz = 1.0 / genome_info.simulation_timestep;
    runtime_service
        .set_frequency(burst_frequency_hz)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to update burst frequency: {}", e)))?;

    if runtime_was_running {
        runtime_service.start().await.map_err(|e| {
            ApiError::internal(format!(
                "Failed to restart burst engine after genome transition: {}",
                e
            ))
        })?;
    }

    tracing::info!(
        target: "feagi-api",
        "✅ Prioritized genome transition completed from {}",
        source
    );

    // Deterministic: create missing IO areas for any registered agents immediately after genome load.
    // Fixes nondeterministic behavior where areas were missing on first run but appeared on restart.
    #[cfg(feature = "feagi-agent")]
    if let Some(handler) = &state.agent_handler {
        let device_regs_list: Vec<serde_json::Value> = {
            let guard = handler.lock().unwrap();
            guard
                .get_all_registered_agents()
                .keys()
                .filter_map(|sid| guard.get_device_registrations_by_agent(*sid).cloned())
                .collect()
        };
        for device_regs in device_regs_list {
            let conflicts = crate::common::agent_registration::registration_dimension_conflicts(
                state,
                &device_regs,
            )
            .await;
            if !conflicts.is_empty() {
                // Registration never resizes areas: leave this agent's areas as the genome has them.
                tracing::warn!(
                    "[API] Skipping device-registration reconcile after genome load: {} area(s) differ from the declared dimensions",
                    conflicts.len()
                );
                continue;
            }
            crate::common::agent_registration::auto_create_cortical_areas_from_device_registrations(
                state,
                &device_regs,
            )
            .await;
        }
    }

    genome_sm_lifecycle.succeed();
    Ok(genome_info)
}

/// Get the current genome file name.
#[utoipa::path(get, path = "/v1/genome/file_name", tag = "genome")]
pub async fn get_file_name(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Get current genome filename
    Ok(Json(HashMap::from([(
        "genome_file_name".to_string(),
        "".to_string(),
    )])))
}

/// Get list of available circuit templates from the circuit library.
#[utoipa::path(get, path = "/v1/genome/circuits", tag = "genome")]
pub async fn get_circuits(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    // TODO: Get available circuit library
    Ok(Json(vec![]))
}

/// Set the destination for genome amalgamation (merging genomes).
#[utoipa::path(post, path = "/v1/genome/amalgamation_destination", tag = "genome")]
pub async fn post_amalgamation_destination(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
    Json(req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // BV sends query params:
    // - circuit_origin_x/y/z
    // - amalgamation_id
    // - rewire_mode
    //
    // Body:
    // - brain_region_id
    let amalgamation_id = params
        .get("amalgamation_id")
        .ok_or_else(|| ApiError::invalid_input("amalgamation_id required"))?
        .to_string();

    let origin_x: i32 = params
        .get("circuit_origin_x")
        .ok_or_else(|| ApiError::invalid_input("circuit_origin_x required"))?
        .parse()
        .map_err(|_| ApiError::invalid_input("circuit_origin_x must be an integer"))?;
    let origin_y: i32 = params
        .get("circuit_origin_y")
        .ok_or_else(|| ApiError::invalid_input("circuit_origin_y required"))?
        .parse()
        .map_err(|_| ApiError::invalid_input("circuit_origin_y must be an integer"))?;
    let origin_z: i32 = params
        .get("circuit_origin_z")
        .ok_or_else(|| ApiError::invalid_input("circuit_origin_z required"))?
        .parse()
        .map_err(|_| ApiError::invalid_input("circuit_origin_z must be an integer"))?;

    let rewire_mode = params
        .get("rewire_mode")
        .cloned()
        .unwrap_or_else(|| "rewire_all".to_string());

    let parent_region_id = req
        .get("brain_region_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::invalid_input("brain_region_id required"))?
        .to_string();

    // Resolve the pending request.
    let pending = {
        let lock = state.amalgamation_state.write();
        let Some(p) = lock.pending.as_ref() else {
            return Err(ApiError::invalid_input("No amalgamation is pending"));
        };
        if p.summary.amalgamation_id != amalgamation_id {
            return Err(ApiError::invalid_input(format!(
                "Pending amalgamation_id mismatch: expected {}, got {}",
                p.summary.amalgamation_id, amalgamation_id
            )));
        }
        p.clone()
    };

    // One genome-changing operation at a time (as genome load and reset).
    let _transition_lock = state.genome_transition_lock.try_write().map_err(|_| {
        ApiError::conflict("Another genome transition is in progress; wait for it to finish")
    })?;

    let request = AmalgamationRequest {
        amalgamation_id,
        origin: (origin_x, origin_y, origin_z),
        rewire_mode,
        parent_region_id,
    };
    let mut created = AmalgamationCreated::default();
    match apply_amalgamation(&state, &pending, &request, &mut created).await {
        Ok(response) => Ok(Json(response)),
        Err(mut error) => {
            let failures = rollback_amalgamation(&state, &created).await;
            if !failures.is_empty() {
                error.message.push_str(&format!(
                    " Rollback could not remove: {}.",
                    failures.join("; ")
                ));
            }
            // The request cannot be retried as it stands: clear it so the caller queues it again.
            {
                let mut lock = state.amalgamation_state.write();
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                lock.history.push(amalgamation::AmalgamationHistoryEntry {
                    amalgamation_id: pending.summary.amalgamation_id.clone(),
                    genome_title: pending.summary.genome_title.clone(),
                    circuit_size: pending.summary.circuit_size,
                    status: "cancelled".to_string(),
                    timestamp_ms: now_ms,
                });
                lock.pending = None;
            }
            Err(error)
        }
    }
}

/// Confirmed destination of a pending amalgamation.
struct AmalgamationRequest {
    amalgamation_id: String,
    origin: (i32, i32, i32),
    rewire_mode: String,
    parent_region_id: String,
}

/// What an amalgamation created, in creation order, so a failure can undo exactly that.
#[derive(Default)]
struct AmalgamationCreated {
    regions: Vec<String>,
    owned_modulators: Vec<String>,
    areas: Vec<String>,
    adopted_modulators: Vec<String>,
    morphologies: Vec<String>,
    classifiers: Vec<String>,
    /// `(src, dst)` of every mapping a fit plan created, so a failure removes exactly those
    /// (including ones whose source is an existing host area).
    plan_mappings: Vec<(String, String)>,
}

pub(crate) fn is_not_found(error: &feagi_services::ServiceError) -> bool {
    matches!(error, feagi_services::ServiceError::NotFound { .. })
}

/// Undo `created` in reverse order. An item that is already gone (a classifier delete removes its
/// areas and circuit region) counts as removed. Returns one line per item that could not be
/// removed; each is also logged.
async fn rollback_amalgamation(state: &ApiState, created: &AmalgamationCreated) -> Vec<String> {
    let connectome = state.connectome_service.as_ref();
    let genome = state.genome_service.as_ref();
    let mut failures: Vec<String> = Vec::new();
    let mut note = |what: String, error: feagi_services::ServiceError| {
        if is_not_found(&error) {
            return;
        }
        tracing::error!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Rollback could not remove {}: {}",
            what,
            error
        );
        failures.push(format!("{what} ({error})"));
    };
    for (src, dst) in created.plan_mappings.iter().rev() {
        if let Err(e) = connectome
            .update_cortical_mapping(src.clone(), dst.clone(), Vec::new())
            .await
        {
            note(format!("mapping {src} -> {dst}"), e);
        }
    }
    for id in created.classifiers.iter().rev() {
        if let Err(e) = connectome.delete_classifier(id).await {
            note(format!("classifier {id}"), e);
        }
    }
    for name in created.adopted_modulators.iter().rev() {
        if let Err(e) = genome.delete_modulator(name, true).await {
            note(format!("modulator '{name}'"), e);
        }
    }
    for id in created.areas.iter().rev() {
        if let Err(e) = connectome.delete_cortical_area(id).await {
            note(format!("cortical area {id}"), e);
        }
    }
    for id in created.morphologies.iter().rev() {
        if let Err(e) = connectome.delete_morphology(id).await {
            note(format!("morphology '{id}'"), e);
        }
    }
    for name in created.owned_modulators.iter().rev() {
        if let Err(e) = genome.delete_modulator(name, true).await {
            note(format!("modulator '{name}'"), e);
        }
    }
    for id in created.regions.iter().rev() {
        if let Err(e) = connectome.delete_brain_region(id).await {
            note(format!("brain region {id}"), e);
        }
    }
    if !created.regions.is_empty() || !created.areas.is_empty() {
        tracing::warn!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Rolled back after a failure: {} plan mappings, {} classifiers, {} areas, {} modulators, {} morphologies, {} regions",
            created.plan_mappings.len(),
            created.classifiers.len(),
            created.areas.len(),
            created.owned_modulators.len() + created.adopted_modulators.len(),
            created.morphologies.len(),
            created.regions.len()
        );
    }
    failures
}

/// Import the pending guest genome into the brain. Records everything it creates in `created`.
async fn apply_amalgamation(
    state: &ApiState,
    pending: &amalgamation::AmalgamationPending,
    request: &AmalgamationRequest,
    created: &mut AmalgamationCreated,
) -> Result<HashMap<String, serde_json::Value>, ApiError> {
    let amalgamation_id = request.amalgamation_id.clone();
    let (origin_x, origin_y, origin_z) = request.origin;
    let rewire_mode = request.rewire_mode.clone();
    let parent_region_id = request.parent_region_id.clone();
    let connectome_service = state.connectome_service.as_ref();

    // 2) Import cortical areas into that region.
    //
    // - Guest **Custom** and **Memory** cortical IDs are remapped to fresh IDs that do not
    //   collide with the host (or with each other), and `cortical_mapping_dst` keys / brain
    //   region membership are updated accordingly. This preserves the full guest circuit instead
    //   of skipping shared template custom areas.
    // - **Core** (`___death`, `___power`, `___fatig`) and **IPU/OPU** IDs are left canonical;
    //   they are not duplicated if they already exist on the host (`skipped_existing_areas`).
    // - We place areas at an offset relative to the chosen origin and set `parent_region_id`.
    let mut imported_genome = feagi_evolutionary::load_genome_from_json(&pending.genome_json)
        .map_err(|e| {
            ApiError::invalid_input(format!(
                "Pending genome payload can no longer be parsed as a genome: {}",
                e
            ))
        })?;

    // Fit before any area is created (a later resize would rebuild the area). The plan was
    // validated and stored when the amalgamation was queued; it names guest ids as published, so
    // this runs before the Custom/Memory id remap below.
    if let Some(plan) = &pending.fit_plan {
        imported_genome.cortical_areas =
            feagi_evolutionary::fit_areas(&imported_genome.cortical_areas, plan)
                .map_err(|e| ApiError::invalid_input(format!("Invalid fit_plan: {}", e)))?;
    }

    // The plan's wiring was validated against the host when it was queued. The host may have
    // changed since: read it again and refuse (nothing is created yet) when a host area the
    // wiring ends on is gone or has another size, or when anything else no longer holds.
    let wiring = pending.fit_plan.as_ref().filter(|plan| plan.has_wiring());
    if let Some(plan) = wiring {
        let guest_morphologies: std::collections::HashSet<String> = imported_genome
            .morphologies
            .morphology_ids()
            .into_iter()
            .collect();
        let host_now = read_plan_host(state, plan, &imported_genome.cortical_areas).await?;
        for (id, expected) in &pending.fit_host_ends {
            match host_now.areas.get(id) {
                Some(now) if now == expected => {}
                Some(now) => {
                    return Err(ApiError::invalid_input(format!(
                        "Host area {} changed size since the plan was queued ({:?} -> {:?}); \
                         queue the block again",
                        id, expected, now
                    )))
                }
                None => {
                    return Err(ApiError::invalid_input(format!(
                        "Host area {} no longer exists; queue the block again",
                        id
                    )))
                }
            }
        }
        check_plan_wiring(
            plan,
            &imported_genome.cortical_areas,
            &guest_morphologies,
            &host_now,
        )?;
        // A mapping out of an existing host area replaces any rule already there, which a
        // rollback could not restore: refuse it.
        let guest_ids: std::collections::HashSet<String> = imported_genome
            .cortical_areas
            .keys()
            .map(|id| id.as_base_64())
            .collect();
        for mapping in &plan.mappings {
            if guest_ids.contains(&mapping.src) || !host_now.areas.contains_key(&mapping.src) {
                continue;
            }
            let properties = connectome_service
                .get_cortical_area_properties(&mapping.src)
                .await
                .map_err(|e| {
                    ApiError::internal(format!("Failed to read area {}: {}", mapping.src, e))
                })?;
            let wired = properties
                .get("cortical_mapping_dst")
                .and_then(|v| v.as_object())
                .is_some_and(|dsts| dsts.contains_key(&mapping.dst));
            if wired {
                return Err(ApiError::invalid_input(format!(
                    "Invalid fit_plan: host mapping {} -> {} already exists",
                    mapping.src, mapping.dst
                )));
            }
        }
    }

    let host_cortical_ids: std::collections::HashSet<String> = connectome_service
        .get_cortical_area_ids()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list cortical area IDs: {}", e)))?
        .into_iter()
        .collect();

    // Adapter ids are not in the genome, so the remap must be told to keep clear of them.
    let mut remap_reserved_ids = host_cortical_ids.clone();
    if let Some(adapter) = wiring.and_then(|plan| plan.adapter.as_ref()) {
        remap_reserved_ids.extend(adapter.areas.iter().map(|a| a.cortical_id.clone()));
    }
    let remapped_guest_custom_memory_ids =
        feagi_evolutionary::remap_guest_custom_memory_cortical_ids_for_amalgamation(
            &mut imported_genome,
            &remap_reserved_ids,
        )
        .map_err(|e| {
            ApiError::internal(format!(
                "Amalgamation guest cortical ID remapping failed: {}",
                e
            ))
        })?;
    let guest_custom_memory_id_remap_count = remapped_guest_custom_memory_ids.len();

    if guest_custom_memory_id_remap_count > 0 {
        tracing::info!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Remapped {} guest Custom/Memory cortical IDs before import",
            guest_custom_memory_id_remap_count
        );
    }

    let genome_service = state.genome_service.as_ref();

    // Plan the guest modulators before anything is created: names, renames and reuse, and the
    // validation that an adopted driver is free. A failure here leaves nothing to undo.
    let host_modulators: HashMap<String, feagi_evolutionary::HostModulator> = genome_service
        .list_modulators()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list modulators: {}", e)))?
        .as_object()
        .map(|object| {
            object
                .iter()
                .filter_map(|(name, body)| {
                    let kind = feagi_structures::genomic::ModulatorKind::parse(
                        body.get("type")?.as_str()?,
                    )
                    .ok()?;
                    Some((
                        name.clone(),
                        feagi_evolutionary::HostModulator {
                            kind,
                            driver_cortical_id: body
                                .get("driver_cortical_id")?
                                .as_str()?
                                .to_string(),
                        },
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let collision_policy = match pending.source {
        amalgamation::AmalgamationSource::SameBrainClone => {
            feagi_evolutionary::ModulatorCollisionPolicy::ReuseHost
        }
        amalgamation::AmalgamationSource::ExternalGenome => {
            feagi_evolutionary::ModulatorCollisionPolicy::RenameGuest {
                block_title: pending.summary.genome_title.clone(),
            }
        }
    };
    let modulator_plan = feagi_evolutionary::plan_amalgamation_modulators(
        &imported_genome,
        &host_modulators,
        &host_cortical_ids,
        &collision_policy,
    )
    .map_err(|e| {
        ApiError::invalid_input(format!("Cannot import the circuit's modulators: {}", e))
    })?;
    feagi_evolutionary::apply_modulator_name_map(&mut imported_genome, &modulator_plan.name_map);
    for notice in &modulator_plan.notices {
        tracing::info!(target: "feagi-api", "🧬 [AMALGAMATION] {}", notice);
    }
    for orphan in &modulator_plan.orphan_driver_ids {
        tracing::warn!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Skipping modulator driver area {}: the imported circuit has no modulator for it",
            orphan
        );
    }

    let mut to_create: Vec<feagi_services::types::CreateCorticalAreaParams> = Vec::new();
    let mut skipped_existing: Vec<String> = Vec::new();

    // Get root region ID for IPU/OPU areas
    let root_region_id = connectome_service
        .get_root_region_id()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get root region ID: {}", e)))?;

    // Pre-pass: which guest areas already exist on the host, and which may live inside the
    // amalgamation region tree. IPU/OPU areas go to root. Modulator driver areas are never
    // created here: the service creates an owned driver together with its modulator, and a
    // reused modulator already has its driver.
    let mut existing_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut tree_member_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for area in imported_genome.cortical_areas.values() {
        let cortical_id = area.cortical_id.as_base_64();
        if area.cortical_id.is_modulator_driver() {
            // A driver id can equal a host driver id (ids are serials), so test the type first.
            continue;
        }
        let exists = connectome_service
            .cortical_area_exists(&cortical_id)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to check existing cortical area {}: {}",
                    cortical_id, e
                ))
            })?;
        if exists {
            existing_ids.insert(cortical_id);
            continue;
        }
        match area.cortical_id.as_cortical_type().map_err(|e| {
            ApiError::internal(format!(
                "Failed to get cortical area type for {}: {}",
                cortical_id, e
            ))
        })? {
            feagi_structures::genomic::cortical_area::CorticalAreaType::BrainInput(_)
            | feagi_structures::genomic::cortical_area::CorticalAreaType::BrainOutput(_) => {}
            feagi_structures::genomic::cortical_area::CorticalAreaType::Modulator => {}
            _ => {
                tree_member_ids.insert(cortical_id);
            }
        }
    }

    // Plan the guest region tree and create sub-regions (parent before child) before any area
    // is created, so each area's `parent_region_id` already exists.
    let region_plan = feagi_evolutionary::plan_amalgamation_regions(
        &imported_genome,
        &amalgamation_id,
        (origin_x, origin_y, origin_z),
        &tree_member_ids,
    );
    // Up-front validation of the classifiers (nothing is created yet). The final plan is made
    // after the modulator drivers have their ids.
    feagi_evolutionary::plan_amalgamation_classifiers(
        &imported_genome,
        &HashMap::new(),
        &modulator_plan.name_map,
        &region_plan.region_map,
        &amalgamation_id,
    )
    .map_err(|e| {
        ApiError::invalid_input(format!("Cannot import the circuit's classifiers: {}", e))
    })?;

    // 1) Create a new brain region to host the imported circuit. Nothing above changed the brain.
    // Note: ConnectomeServiceImpl shares the same RuntimeGenome Arc with GenomeServiceImpl, so
    // persisting the region into the RuntimeGenome is required for subsequent cortical-area creation.
    let mut region_properties: HashMap<String, serde_json::Value> = HashMap::new();
    region_properties.insert(
        "coordinate_3d".to_string(),
        serde_json::json!([origin_x, origin_y, origin_z]),
    );
    region_properties.insert(
        "amalgamation_id".to_string(),
        serde_json::json!(pending.summary.amalgamation_id),
    );
    region_properties.insert(
        "circuit_size".to_string(),
        serde_json::json!(pending.summary.circuit_size),
    );
    region_properties.insert("rewire_mode".to_string(), serde_json::json!(rewire_mode));

    connectome_service
        .create_brain_region(feagi_services::types::CreateBrainRegionParams {
            region_id: amalgamation_id.clone(),
            name: pending.summary.genome_title.clone(),
            region_type: "Custom".to_string(),
            parent_id: Some(parent_region_id.clone()),
            properties: Some(region_properties),
        })
        .await
        .map_err(|e| {
            ApiError::internal(format!("Failed to create amalgamation brain region: {}", e))
        })?;
    created.regions.push(amalgamation_id.clone());

    for spec in &region_plan.sub_regions {
        connectome_service
            .create_brain_region(feagi_services::types::CreateBrainRegionParams {
                region_id: spec.region_id.clone(),
                name: spec.name.clone(),
                region_type: "Custom".to_string(),
                parent_id: Some(spec.parent_region_id.clone()),
                properties: Some(spec.properties.clone()),
            })
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to create amalgamation sub-region '{}': {}",
                    spec.name, e
                ))
            })?;
        created.regions.push(spec.region_id.clone());
    }
    let created_sub_region_count = region_plan.sub_regions.len();
    tracing::info!(
        target: "feagi-api",
        "🧬 [AMALGAMATION] Created {} guest sub-regions under region {}",
        created_sub_region_count,
        amalgamation_id
    );

    // The plan's adapter gets its own region next to the block's (never inside it), so the block
    // can be packaged without it. Its areas are created right after the guest areas below.
    let adapter_plan = wiring.and_then(|plan| plan.adapter.as_ref());
    let mut adapter_region_id: Option<String> = None;
    if let Some(adapter) = adapter_plan {
        let region_id = Uuid::new_v4().to_string();
        let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
        properties.insert(
            "coordinate_3d".to_string(),
            serde_json::json!([
                origin_x
                    .saturating_add(pending.summary.circuit_size[0])
                    .saturating_add(4),
                origin_y,
                origin_z
            ]),
        );
        properties.insert(
            "amalgamation_id".to_string(),
            serde_json::json!(pending.summary.amalgamation_id),
        );
        connectome_service
            .create_brain_region(feagi_services::types::CreateBrainRegionParams {
                region_id: region_id.clone(),
                name: adapter
                    .region_name
                    .clone()
                    .unwrap_or_else(|| format!("{} adapter", pending.summary.genome_title)),
                region_type: "Custom".to_string(),
                parent_id: Some(parent_region_id.clone()),
                properties: Some(properties),
            })
            .await
            .map_err(|e| {
                ApiError::internal(format!("Failed to create adapter brain region: {}", e))
            })?;
        created.regions.push(region_id.clone());
        adapter_region_id = Some(region_id);
    }

    // Own modulators: created through the service, which allocates each driver id (a guest
    // driver id is a serial that can collide with a host one). Then every reference to a guest
    // driver is rewritten through the resulting table.
    let mut driver_remap: HashMap<String, String> = modulator_plan.reused_driver_remap.clone();
    for owned in &modulator_plan.owned {
        let body = genome_service
            .create_modulator(owned.host_name.clone(), owned.write.clone())
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to create modulator '{}': {}",
                    owned.host_name, e
                ))
            })?;
        created.owned_modulators.push(owned.host_name.clone());
        let new_driver = body
            .get("driver_cortical_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ApiError::internal(format!(
                    "Modulator '{}' was created without a driver area id",
                    owned.host_name
                ))
            })?
            .to_string();
        // The service creates a driver with default tuning; carry the guest's.
        if !owned.driver_tuning.is_empty() {
            if let Err(e) = genome_service
                .update_cortical_area(&new_driver, owned.driver_tuning.clone())
                .await
            {
                tracing::warn!(
                    target: "feagi-api",
                    "🧬 [AMALGAMATION] Modulator '{}' driver tuning was not copied: {}",
                    owned.host_name,
                    e
                );
            }
        }
        driver_remap.insert(owned.guest_driver_id.clone(), new_driver);
    }
    feagi_evolutionary::apply_driver_remap(&mut imported_genome, &driver_remap).map_err(|e| {
        ApiError::internal(format!(
            "Amalgamation modulator driver remapping failed: {}",
            e
        ))
    })?;

    for area in imported_genome.cortical_areas.values() {
        let cortical_id = area.cortical_id.as_base_64();
        if existing_ids.contains(&cortical_id) {
            skipped_existing.push(cortical_id);
            continue;
        }
        if area.cortical_id.is_modulator_driver() {
            continue;
        }

        let mut props = area.properties.clone();

        // Remove cortical_mapping_dst from properties - connections will be imported separately
        props.remove("cortical_mapping_dst");

        // Determine correct parent region based on area type
        // IPU/OPU areas MUST go to root region, all others go to the amalgamation region
        let area_type = area.cortical_id.as_cortical_type().map_err(|e| {
            ApiError::internal(format!(
                "Failed to get cortical area type for {}: {}",
                cortical_id, e
            ))
        })?;

        let target_parent_region_id = match area_type {
            feagi_structures::genomic::cortical_area::CorticalAreaType::BrainInput(_)
            | feagi_structures::genomic::cortical_area::CorticalAreaType::BrainOutput(_) => {
                // IPU/OPU areas go to root region
                match root_region_id.as_ref() {
                    Some(root_id) => {
                        tracing::info!(
                            target: "feagi-api",
                            "🧬 [AMALGAMATION] IPU/OPU area {} will be placed in root region {}",
                            cortical_id,
                            root_id
                        );
                        root_id.clone()
                    }
                    None => {
                        tracing::warn!(
                            target: "feagi-api",
                            "🧬 [AMALGAMATION] No root region found for IPU/OPU area {}, using amalgamation region",
                            cortical_id
                        );
                        amalgamation_id.clone()
                    }
                }
            }
            _ => {
                // Custom, Memory, Core areas go to their planned guest sub-region, or to the
                // amalgamation region when the guest lists them in no region.
                region_plan
                    .area_region
                    .get(&cortical_id)
                    .cloned()
                    .unwrap_or_else(|| amalgamation_id.clone())
            }
        };

        props.insert(
            "parent_region_id".to_string(),
            serde_json::json!(target_parent_region_id),
        );
        props.insert(
            "amalgamation_source".to_string(),
            serde_json::json!("amalgamation_by_payload"),
        );

        to_create.push(feagi_services::types::CreateCorticalAreaParams {
            cortical_id,
            name: area.name.clone(),
            dimensions: (
                area.dimensions.width as usize,
                area.dimensions.height as usize,
                area.dimensions.depth as usize,
            ),
            position: (
                origin_x.saturating_add(area.position.x),
                origin_y.saturating_add(area.position.y),
                origin_z.saturating_add(area.position.z),
            ),
            area_type: "Custom".to_string(),
            visible: Some(true),
            sub_group: None,
            neurons_per_voxel: area
                .properties
                .get("neurons_per_voxel")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            postsynaptic_current: area
                .properties
                .get("postsynaptic_current")
                .and_then(|v| v.as_f64()),
            plasticity_constant: area
                .properties
                .get("plasticity_constant")
                .and_then(|v| v.as_f64()),
            degeneration: area.properties.get("degeneration").and_then(|v| v.as_f64()),
            psp_uniform_distribution: area
                .properties
                .get("psp_uniform_distribution")
                .and_then(|v| v.as_bool()),
            firing_threshold_increment: None,
            firing_threshold_limit: area
                .properties
                .get("firing_threshold_limit")
                .and_then(|v| v.as_f64()),
            consecutive_fire_count: area
                .properties
                .get("consecutive_fire_limit")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            snooze_period: area
                .properties
                .get("snooze_period")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            refractory_period: area
                .properties
                .get("refractory_period")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            leak_coefficient: area
                .properties
                .get("leak_coefficient")
                .and_then(|v| v.as_f64()),
            leak_variability: area
                .properties
                .get("leak_variability")
                .and_then(|v| v.as_f64()),
            burst_engine_active: area
                .properties
                .get("burst_engine_active")
                .and_then(|v| v.as_bool()),
            properties: Some(props),
        });
    }

    let imported_new_area_count = to_create.len();
    // Area subscriptions (`modulators`) are stored with the area but only reach the burst engine
    // through a modulator sync, which area creation does not run. Re-apply them below, once every
    // modulator exists.
    let area_subscriptions: Vec<(String, Vec<serde_json::Value>)> = to_create
        .iter()
        .filter_map(|params| {
            let list = params
                .properties
                .as_ref()?
                .get("modulators")?
                .as_array()?
                .clone();
            (!list.is_empty()).then(|| (params.cortical_id.clone(), list))
        })
        .collect();
    created
        .areas
        .extend(to_create.iter().map(|params| params.cortical_id.clone()));
    if !to_create.is_empty() {
        genome_service
            .create_cortical_areas(to_create)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to import cortical areas: {}", e)))?;
    }

    // Adapter areas, in the adapter region, at their planned sizes. Plain defaults; the plan
    // names no neuron settings.
    let mut adapter_area_count = 0_usize;
    if let (Some(adapter), Some(region_id)) = (adapter_plan, adapter_region_id.as_ref()) {
        let mut x_offset = origin_x
            .saturating_add(pending.summary.circuit_size[0])
            .saturating_add(4);
        let mut params: Vec<feagi_services::types::CreateCorticalAreaParams> = Vec::new();
        for area in &adapter.areas {
            let [w, h, d] = area.dimensions;
            params.push(feagi_services::types::CreateCorticalAreaParams {
                cortical_id: area.cortical_id.clone(),
                name: area.name.clone(),
                dimensions: (w as usize, h as usize, d as usize),
                position: (x_offset, origin_y, origin_z),
                area_type: "Custom".to_string(),
                visible: Some(true),
                sub_group: None,
                neurons_per_voxel: None,
                postsynaptic_current: None,
                plasticity_constant: None,
                degeneration: None,
                psp_uniform_distribution: None,
                firing_threshold_increment: None,
                firing_threshold_limit: None,
                consecutive_fire_count: None,
                snooze_period: None,
                refractory_period: None,
                leak_coefficient: None,
                leak_variability: None,
                burst_engine_active: None,
                properties: Some(HashMap::from([
                    ("parent_region_id".to_string(), serde_json::json!(region_id)),
                    (
                        "amalgamation_source".to_string(),
                        serde_json::json!("amalgamation_by_payload"),
                    ),
                ])),
            });
            x_offset = x_offset.saturating_add(w as i32).saturating_add(2);
        }
        adapter_area_count = params.len();
        created
            .areas
            .extend(params.iter().map(|p| p.cortical_id.clone()));
        genome_service
            .create_cortical_areas(params)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to create adapter areas: {}", e)))?;
    }

    // Adopting modulators: their driver areas exist now.
    for adopted in &modulator_plan.adopted {
        genome_service
            .create_modulator(adopted.host_name.clone(), adopted.write.clone())
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to create modulator '{}' on area {}: {}",
                    adopted.host_name, adopted.driver_cortical_id, e
                ))
            })?;
        created.adopted_modulators.push(adopted.host_name.clone());
    }
    for (area_id, list) in area_subscriptions {
        genome_service
            .update_cortical_area(
                &area_id,
                HashMap::from([("modulators".to_string(), serde_json::Value::Array(list))]),
            )
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to subscribe imported area {} to its modulators: {}",
                    area_id, e
                ))
            })?;
    }
    let imported_modulator_count =
        created.owned_modulators.len() + created.adopted_modulators.len();

    // 3) Import morphologies used by the imported areas' cortical mappings.
    //
    // Collect all morphology IDs referenced in the cortical_mapping_dst of imported areas,
    // then import those morphologies from the imported genome into the current genome.
    let imported_area_ids: std::collections::HashSet<String> = imported_genome
        .cortical_areas
        .keys()
        .filter(|id| !id.is_modulator_driver())
        .map(|id| id.as_base_64())
        .filter(|id| !skipped_existing.contains(id))
        .collect();

    let mut required_morphologies: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    // Scan imported areas' mappings to collect required morphology IDs
    for area in imported_genome.cortical_areas.values() {
        if !imported_area_ids.contains(&area.cortical_id.as_base_64()) {
            continue;
        }

        let Some(cortical_mapping_dst) = area.properties.get("cortical_mapping_dst") else {
            continue;
        };
        let Some(dst_map) = cortical_mapping_dst.as_object() else {
            continue;
        };

        for mapping_data in dst_map.values() {
            let Some(mapping_array) = mapping_data.as_array() else {
                continue;
            };

            for rule in mapping_array {
                // Extract morphology_id from rule (can be object or array format)
                let morphology_id = if let Some(obj) = rule.as_object() {
                    obj.get("morphology_id").and_then(|v| v.as_str())
                } else if let Some(arr) = rule.as_array() {
                    arr.first().and_then(|v| v.as_str())
                } else {
                    None
                };

                if let Some(morph_id) = morphology_id {
                    required_morphologies.insert(morph_id.to_string());
                }
            }
        }
    }

    // Plan rules may use a guest morphology the guest's own mappings do not.
    if let Some(plan) = wiring {
        let defined: std::collections::HashSet<&str> =
            plan.morphologies.iter().map(|m| m.name.as_str()).collect();
        for rule in plan.mappings.iter().flat_map(|m| m.rules.iter()) {
            if let Some(name) = rule.get("morphology_id").and_then(|v| v.as_str()) {
                if !defined.contains(name) {
                    required_morphologies.insert(name.to_string());
                }
            }
        }
    }

    // Import each required morphology if it doesn't already exist
    let mut imported_morphology_count = 0;
    let mut skipped_morphology_count = 0;

    for morphology_id in &required_morphologies {
        // Check if morphology already exists
        let morphologies = connectome_service.get_morphologies().await.map_err(|e| {
            ApiError::internal(format!("Failed to get existing morphologies: {}", e))
        })?;

        if morphologies.contains_key(morphology_id) {
            skipped_morphology_count += 1;
            continue;
        }

        // Get morphology from imported genome
        let Some(morphology) = imported_genome.morphologies.get(morphology_id) else {
            tracing::warn!(
                target: "feagi-api",
                "🧬 [AMALGAMATION] Morphology '{}' referenced in mappings but not found in imported genome",
                morphology_id
            );
            continue;
        };

        // Import the morphology
        match connectome_service
            .create_morphology(morphology_id.clone(), morphology.clone())
            .await
        {
            Ok(_) => {
                tracing::debug!(
                    target: "feagi-api",
                    "🧬 [AMALGAMATION] Imported morphology '{}'",
                    morphology_id
                );
                created.morphologies.push(morphology_id.clone());
                imported_morphology_count += 1;
            }
            Err(e) => {
                tracing::warn!(
                    target: "feagi-api",
                    "🧬 [AMALGAMATION] Failed to import morphology '{}': {}",
                    morphology_id,
                    e
                );
            }
        }
    }

    if imported_morphology_count > 0 {
        tracing::info!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Imported {} morphologies (skipped {} existing)",
            imported_morphology_count,
            skipped_morphology_count
        );
    }

    // 4) Import cortical mappings from the guest genome.
    //
    // **Source** must be a newly created area (in `imported_area_ids`). Sources that were skipped
    // due to ID collision never have their outgoing mappings applied here.
    // **Destination** must already exist in the connectome (newly created or pre-existing host area).
    // So: new→new and new→host edges can be imported; skipped→* edges are not.

    let mut imported_mapping_count = 0;
    let mut skipped_mapping_count = 0;

    for area in imported_genome.cortical_areas.values() {
        let src_area_id = area.cortical_id.as_base_64();

        // Skip if this area was not imported (already existed)
        if !imported_area_ids.contains(&src_area_id) {
            continue;
        }

        // Check if area has cortical_mapping_dst property
        let Some(cortical_mapping_dst) = area.properties.get("cortical_mapping_dst") else {
            continue;
        };
        let Some(dst_map) = cortical_mapping_dst.as_object() else {
            continue;
        };

        // Import each mapping where destination exists in connectome
        for (dst_area_id, mapping_data) in dst_map {
            // Check if destination area exists in connectome (either newly imported or already existing)
            let dst_exists = connectome_service
                .cortical_area_exists(dst_area_id)
                .await
                .unwrap_or(false);

            if !dst_exists {
                // Skip external references to areas not in this brain
                skipped_mapping_count += 1;
                continue;
            }

            let Some(mapping_array) = mapping_data.as_array() else {
                tracing::warn!(
                    target: "feagi-api",
                    "🧬 [AMALGAMATION] Invalid mapping data from {} to {}: not an array",
                    src_area_id,
                    dst_area_id
                );
                continue;
            };

            // Import the cortical mapping
            match connectome_service
                .update_cortical_mapping(
                    src_area_id.clone(),
                    dst_area_id.clone(),
                    mapping_array.clone(),
                )
                .await
            {
                Ok(synapse_count) => {
                    tracing::debug!(
                        target: "feagi-api",
                        "[AMALGAMATION] Imported mapping {} -> {} ({} synapses)",
                        src_area_id,
                        dst_area_id,
                        synapse_count
                    );
                    imported_mapping_count += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        target: "feagi-api",
                        "[AMALGAMATION] Failed to import mapping {} -> {}: {}",
                        src_area_id,
                        dst_area_id,
                        e
                    );
                    skipped_mapping_count += 1;
                }
            }
        }
    }

    if imported_mapping_count > 0 {
        tracing::info!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Successfully imported {} cortical mappings (skipped {} external/missing mappings)",
            imported_mapping_count,
            skipped_mapping_count
        );
    } else if skipped_mapping_count > 0 {
        tracing::warn!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] No internal mappings imported! Skipped {} mappings (all external or missing)",
            skipped_mapping_count
        );
    } else {
        // Neither imported nor skipped counts: usually empty/missing `cortical_mapping_dst` on
        // **newly created** guest areas after parse (e.g. flat blueprint `dstmap-d` is `{}`).
        let mut nonempty_dst_on_new = 0_usize;
        let mut empty_dst_on_new = 0_usize;
        let mut missing_dst_on_new = 0_usize;
        for area in imported_genome.cortical_areas.values() {
            let id = area.cortical_id.as_base_64();
            if !imported_area_ids.contains(&id) {
                continue;
            }
            match area.properties.get("cortical_mapping_dst") {
                None => missing_dst_on_new += 1,
                Some(v) => {
                    if v.as_object().map(|o| o.is_empty()).unwrap_or(true) {
                        empty_dst_on_new += 1;
                    } else {
                        nonempty_dst_on_new += 1;
                    }
                }
            }
        }
        tracing::info!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] No cortical mappings to import from guest (new areas: nonempty_dst={} empty_dst={} missing_dst={}; imported_new_areas={}; skipped_existing_areas={}). \
             Areas skipped due to host ID collision do not replay guest wiring. \
             For synapses on newly created areas, guest blueprint needs non-empty dstmap (cortical_mapping_dst) on those sources.",
            nonempty_dst_on_new,
            empty_dst_on_new,
            missing_dst_on_new,
            imported_new_area_count,
            skipped_existing.len()
        );
    }

    // 4a) The plan's morphologies and mappings. Unlike the guest's own wiring above, a failure
    // here fails the whole import (rolled back by the caller): the block is only usable with it.
    let mut plan_morphology_count = 0_usize;
    let mut plan_mapping_reports: Vec<serde_json::Value> = Vec::new();
    if let Some(plan) = wiring {
        for morphology in &plan.morphologies {
            connectome_service
                .create_morphology(morphology.name.clone(), morphology.to_morphology())
                .await
                .map_err(|e| {
                    ApiError::internal(format!(
                        "Failed to create plan morphology '{}': {}",
                        morphology.name, e
                    ))
                })?;
            created.morphologies.push(morphology.name.clone());
            plan_morphology_count += 1;
        }
        // Guest ids in the plan are as published; the import renamed its custom and memory areas.
        let live_id = |id: &str| {
            remapped_guest_custom_memory_ids
                .get(id)
                .cloned()
                .unwrap_or_else(|| id.to_string())
        };
        for mapping in &plan.mappings {
            let (src, dst) = (live_id(&mapping.src), live_id(&mapping.dst));
            let synapse_count = connectome_service
                .update_cortical_mapping(src.clone(), dst.clone(), mapping.rules.clone())
                .await
                .map_err(|e| {
                    ApiError::internal(format!("Failed to create plan mapping {src} -> {dst}: {e}"))
                })?;
            created.plan_mappings.push((src.clone(), dst.clone()));
            plan_mapping_reports.push(serde_json::json!({
                "src": src,
                "dst": dst,
                "synapse_count": synapse_count,
            }));
        }
    }

    // 4b) Classifiers last: their areas, reward modulators and mappings exist now. Each gets a
    // fresh id (the service overwrites a record with the same id). Area ids are already final in
    // the guest genome, so no further area remap is passed.
    let planned_classifiers = feagi_evolutionary::plan_amalgamation_classifiers(
        &imported_genome,
        &HashMap::new(),
        &modulator_plan.name_map,
        &region_plan.region_map,
        &amalgamation_id,
    )
    .map_err(|e| ApiError::internal(format!("Cannot import the circuit's classifiers: {}", e)))?;
    let imported_classifier_count = planned_classifiers.len();
    for classifier in planned_classifiers {
        let classifier_id = classifier.classifier_id.clone();
        let classifier_name = classifier.name.clone();
        connectome_service
            .upsert_classifier(classifier)
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to import classifier '{}': {}",
                    classifier_name, e
                ))
            })?;
        created.classifiers.push(classifier_id);
    }

    // Copy the source circuit's declared input/output roles onto the new region.
    // Cross-region edges are stripped on export, so connectivity alone does not restore them.
    let designated_inputs = region_plan.host_designated_inputs.clone();
    let designated_outputs = region_plan.host_designated_outputs.clone();
    let port_encoding = region_plan.host_port_encoding.clone();
    if !designated_inputs.is_empty() || !designated_outputs.is_empty() {
        let mut io_properties: HashMap<String, serde_json::Value> = HashMap::new();
        io_properties.insert(
            "designated_inputs".to_string(),
            serde_json::json!(designated_inputs),
        );
        io_properties.insert(
            "designated_outputs".to_string(),
            serde_json::json!(designated_outputs),
        );
        if let Err(error) = connectome_service
            .update_brain_region(&amalgamation_id, io_properties)
            .await
        {
            tracing::warn!(
                target: "feagi-api",
                "Amalgamation created region {} but could not copy region IO designations: {}",
                amalgamation_id,
                error
            );
        } else if !port_encoding.is_empty() {
            // Internal import path: entries keep their source (`wiring` stays `wiring`, `author`
            // stays `author`). The public region update would store every entry as `author`.
            if let Err(error) = connectome_service
                .import_region_port_encoding(&amalgamation_id, port_encoding)
                .await
            {
                tracing::warn!(
                    target: "feagi-api",
                    "Amalgamation created region {} but could not copy its port encoding: {}",
                    amalgamation_id,
                    error
                );
            }
        }
    }

    // Record how the block's published area ids map to the live ones (and the Hub release it came
    // from, when the caller named one) so later wiring can find the block's areas by their
    // published ids. Stored on the block region, so it is saved with the genome.
    if !remapped_guest_custom_memory_ids.is_empty()
        || pending.source_release_id.is_some()
        || pending.source_catalog.is_some()
    {
        let mut stamp: HashMap<String, serde_json::Value> = HashMap::new();
        stamp.insert(
            "published_to_live".to_string(),
            serde_json::json!(remapped_guest_custom_memory_ids),
        );
        if let Some(release) = &pending.source_release_id {
            stamp.insert("source_release_id".to_string(), serde_json::json!(release));
        }
        if let Some(catalog) = &pending.source_catalog {
            stamp.insert("source_catalog".to_string(), serde_json::json!(catalog));
        }
        connectome_service
            .update_brain_region(&amalgamation_id, stamp)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to stamp the block region: {}", e)))?;
    }

    // 5) Invalidate all relevant health_check hashes to force BV cache refresh.
    //
    // Amalgamation modifies:
    // - Brain regions (new region created)
    // - Cortical areas (new areas added)
    // - Brain geometry (positions of new areas)
    // - Morphologies (new morphologies imported)
    // - Cortical mappings (new connections created)
    //
    // Incrementing these hashes signals BV to refresh its cached data without requiring a restart.
    {
        let state_manager = feagi_state_manager::StateManager::instance();
        let state_manager = state_manager.read();

        // Increment each relevant hash (adding 1 invalidates client cache)
        state_manager
            .set_brain_regions_hash(state_manager.get_brain_regions_hash().wrapping_add(1));
        state_manager
            .set_cortical_areas_hash(state_manager.get_cortical_areas_hash().wrapping_add(1));
        state_manager
            .set_brain_geometry_hash(state_manager.get_brain_geometry_hash().wrapping_add(1));
        if imported_morphology_count > 0 || plan_morphology_count > 0 {
            state_manager
                .set_morphologies_hash(state_manager.get_morphologies_hash().wrapping_add(1));
        }
        if imported_mapping_count > 0 || !plan_mapping_reports.is_empty() {
            state_manager.set_cortical_mappings_hash(
                state_manager.get_cortical_mappings_hash().wrapping_add(1),
            );
        }

        tracing::info!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Invalidated health_check hashes for BV cache refresh"
        );
    }

    // Clear pending + write history entry
    {
        let mut lock = state.amalgamation_state.write();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        lock.history.push(amalgamation::AmalgamationHistoryEntry {
            amalgamation_id: pending.summary.amalgamation_id.clone(),
            genome_title: pending.summary.genome_title.clone(),
            circuit_size: pending.summary.circuit_size,
            status: "confirmed".to_string(),
            timestamp_ms: now_ms,
        });
        lock.pending = None;
    }

    let guest_cortical_area_count = imported_genome.cortical_areas.len();

    // Build a BV-compatible list response for brain regions (regions_members-like data, but as list).
    let regions = state
        .connectome_service
        .list_brain_regions()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list brain regions: {}", e)))?;

    let post_merge_brain_region_count = regions.len();
    let post_merge_cortical_area_total = state
        .connectome_service
        .get_cortical_area_ids()
        .await
        .map(|ids| ids.len())
        .map_err(|e| {
            ApiError::internal(format!(
                "Failed to count cortical areas after amalgamation: {}",
                e
            ))
        })?;

    let mut brain_regions: Vec<serde_json::Value> = Vec::new();
    for region in regions {
        // Shape matches BV expectations in FEAGIRequests.gd
        let coordinate_3d = region
            .properties
            .get("coordinate_3d")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([0, 0, 0]));
        let coordinate_2d = region
            .properties
            .get("coordinate_2d")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([0, 0]));

        brain_regions.push(serde_json::json!({
            "region_id": region.region_id,
            "title": region.name,
            "description": "",
            "parent_region_id": region.parent_id,
            "coordinate_2d": coordinate_2d,
            "coordinate_3d": coordinate_3d,
            "areas": region.cortical_areas,
            "regions": region.child_regions,
            "inputs": region.properties.get("inputs").cloned().unwrap_or_else(|| serde_json::json!([])),
            "outputs": region.properties.get("outputs").cloned().unwrap_or_else(|| serde_json::json!([])),
            "designated_inputs": region.properties.get("designated_inputs").cloned().unwrap_or_else(|| serde_json::json!([])),
            "designated_outputs": region.properties.get("designated_outputs").cloned().unwrap_or_else(|| serde_json::json!([])),
            "port_encoding": crate::endpoints::region::port_encoding_json(&region.properties, &region.cortical_areas),
        }));
    }

    tracing::info!(
        target: "feagi-api",
        "🧬 [AMALGAMATION] Complete genome_title='{}' amalgamation_id={} \
         guest_custom_memory_ids_remapped={} guest_cortical_areas={} new_cortical_areas_created={} cortical_areas_skipped_host_collision={} \
         morphologies_imported={} morphologies_skipped_already_present={} \
         cortical_mapping_rules_imported={} cortical_mapping_rules_skipped_unresolved_dst={} \
         post_merge_brain_regions={} post_merge_cortical_areas_total={}",
        pending.summary.genome_title,
        pending.summary.amalgamation_id,
        guest_custom_memory_id_remap_count,
        guest_cortical_area_count,
        imported_new_area_count,
        skipped_existing.len(),
        imported_morphology_count,
        skipped_morphology_count,
        imported_mapping_count,
        skipped_mapping_count,
        post_merge_brain_region_count,
        post_merge_cortical_area_total,
    );

    let mut response = HashMap::from([
        (
            "message".to_string(),
            serde_json::Value::String("Amalgamation confirmed".to_string()),
        ),
        (
            "brain_regions".to_string(),
            serde_json::Value::Array(brain_regions),
        ),
        (
            "skipped_existing_areas".to_string(),
            serde_json::json!(skipped_existing),
        ),
        (
            "imported_new_area_count".to_string(),
            serde_json::json!(imported_new_area_count),
        ),
        (
            "guest_cortical_area_count".to_string(),
            serde_json::json!(guest_cortical_area_count),
        ),
        (
            "guest_custom_memory_id_remap_count".to_string(),
            serde_json::json!(guest_custom_memory_id_remap_count),
        ),
        (
            "imported_cortical_mappings".to_string(),
            serde_json::json!(imported_mapping_count),
        ),
        (
            "skipped_cortical_mappings".to_string(),
            serde_json::json!(skipped_mapping_count),
        ),
        (
            "imported_morphology_count".to_string(),
            serde_json::json!(imported_morphology_count),
        ),
        (
            "skipped_morphology_existing_count".to_string(),
            serde_json::json!(skipped_morphology_count),
        ),
        (
            "created_sub_region_count".to_string(),
            serde_json::json!(created_sub_region_count),
        ),
        (
            "modulator_notices".to_string(),
            serde_json::json!(modulator_plan.notices),
        ),
        (
            "imported_modulator_count".to_string(),
            serde_json::json!(imported_modulator_count),
        ),
        (
            "imported_classifier_count".to_string(),
            serde_json::json!(imported_classifier_count),
        ),
        (
            "post_merge_brain_region_count".to_string(),
            serde_json::json!(post_merge_brain_region_count),
        ),
        (
            "post_merge_cortical_area_total".to_string(),
            serde_json::json!(post_merge_cortical_area_total),
        ),
    ]);
    // Plan wiring is reported only when the plan had any, so a plain import answers as before.
    if wiring.is_some() {
        response.insert(
            "plan_adapter_area_count".to_string(),
            serde_json::json!(adapter_area_count),
        );
        response.insert(
            "plan_morphology_count".to_string(),
            serde_json::json!(plan_morphology_count),
        );
        response.insert(
            "plan_mapping_count".to_string(),
            serde_json::json!(plan_mapping_reports.len()),
        );
        response.insert(
            "plan_mappings".to_string(),
            serde_json::Value::Array(plan_mapping_reports),
        );
        if let Some(region_id) = adapter_region_id {
            response.insert(
                "plan_adapter_region_id".to_string(),
                serde_json::json!(region_id),
            );
        }
    }
    Ok(response)
}

/// Cancel a pending genome amalgamation operation.
#[utoipa::path(delete, path = "/v1/genome/amalgamation_cancellation", tag = "genome")]
pub async fn delete_amalgamation_cancellation(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let mut lock = state.amalgamation_state.write();
    if let Some(pending) = lock.pending.take() {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        lock.history.push(amalgamation::AmalgamationHistoryEntry {
            amalgamation_id: pending.summary.amalgamation_id,
            genome_title: pending.summary.genome_title,
            circuit_size: pending.summary.circuit_size,
            status: "cancelled".to_string(),
            timestamp_ms: now_ms,
        });

        tracing::info!(
            target: "feagi-api",
            "🧬 [AMALGAMATION] Cancelled and cleared pending amalgamation id={}",
            lock.history
                .last()
                .map(|e| e.amalgamation_id.clone())
                .unwrap_or_else(|| "<unknown>".to_string())
        );
    }
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Amalgamation cancelled".to_string(),
    )])))
}

/// Append additional structures to the current genome.
#[utoipa::path(post, path = "/v1/feagi/genome/append", tag = "genome")]
pub async fn post_genome_append(
    State(_state): State<ApiState>,
    Json(_req): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Err(ApiError::internal("Not yet implemented"))
}

/// Load the minimal barebones genome with only essential neural structures.
#[utoipa::path(
    post,
    path = "/v1/genome/upload/barebones",
    responses(
        (status = 200, description = "Barebones genome loaded successfully"),
        (status = 500, description = "Failed to load genome")
    ),
    tag = "genome"
)]
pub async fn post_upload_barebones_genome(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    tracing::debug!(target: "feagi-api", "📥 POST /v1/genome/upload/barebones - Request received");
    let result = load_default_genome(state, "barebones").await;
    match &result {
        Ok(_) => {
            tracing::debug!(target: "feagi-api", "✅ POST /v1/genome/upload/barebones - Success")
        }
        Err(e) => {
            tracing::error!(target: "feagi-api", "❌ POST /v1/genome/upload/barebones - Error: {:?}", e)
        }
    }
    result
}

/// Load the essential genome with core sensory and motor areas.
#[utoipa::path(
    post,
    path = "/v1/genome/upload/essential",
    responses(
        (status = 200, description = "Essential genome loaded successfully"),
        (status = 500, description = "Failed to load genome")
    ),
    tag = "genome"
)]
pub async fn post_upload_essential_genome(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    load_default_genome(state, "essential").await
}

/// Helper function to load a default genome by name from embedded Rust genomes
async fn load_default_genome(
    state: ApiState,
    genome_name: &str,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    tracing::info!(target: "feagi-api", "🔄 Loading {} genome from embedded Rust genomes", genome_name);
    tracing::debug!(target: "feagi-api", "   State components available: genome_service=true, runtime_service=true");
    // Load genome from embedded Rust templates (no file I/O!)
    let genome_json = match genome_name {
        "barebones" => feagi_evolutionary::BAREBONES_GENOME_JSON,
        "essential" => feagi_evolutionary::ESSENTIAL_GENOME_JSON,
        "test" => feagi_evolutionary::TEST_GENOME_JSON,
        "vision" => feagi_evolutionary::VISION_GENOME_JSON,
        _ => {
            return Err(ApiError::invalid_input(format!(
                "Unknown genome name '{}'. Available: barebones, essential, test, vision",
                genome_name
            )))
        }
    };

    tracing::info!(target: "feagi-api","Using embedded {} genome ({} bytes), starting conversion...",
                   genome_name, genome_json.len());

    let params = LoadGenomeParams {
        json_str: genome_json.to_string(),
    };

    tracing::info!(target: "feagi-api","Calling prioritized genome transition loader...");
    let genome_info = load_genome_with_priority(&state, params, "default_genome_endpoint").await?;

    tracing::info!(target: "feagi-api","Successfully loaded {} genome: {} cortical areas, {} brain regions",
               genome_name, genome_info.cortical_area_count, genome_info.brain_region_count);

    // Return response matching Python format
    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::Value::Bool(true));
    response.insert(
        "message".to_string(),
        serde_json::Value::String(format!("{} genome loaded successfully", genome_name)),
    );
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::Value::Number(genome_info.cortical_area_count.into()),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::Value::Number(genome_info.brain_region_count.into()),
    );
    response.insert(
        "genome_id".to_string(),
        serde_json::Value::String(genome_info.genome_id),
    );
    response.insert(
        "genome_title".to_string(),
        serde_json::Value::String(genome_info.genome_title),
    );

    Ok(Json(response))
}

/// Get the current genome name.
#[utoipa::path(
    get,
    path = "/v1/genome/name",
    tag = "genome",
    responses(
        (status = 200, description = "Genome name", body = String)
    )
)]
pub async fn get_name(State(_state): State<ApiState>) -> ApiResult<Json<String>> {
    // Get genome metadata to extract name
    // TODO: Implement proper genome name retrieval from genome service
    Ok(Json("default_genome".to_string()))
}

/// Get the genome creation or modification timestamp.
#[utoipa::path(
    get,
    path = "/v1/genome/timestamp",
    tag = "genome",
    responses(
        (status = 200, description = "Genome timestamp", body = i64)
    )
)]
pub async fn get_timestamp(State(_state): State<ApiState>) -> ApiResult<Json<i64>> {
    // TODO: Store and retrieve genome timestamp
    Ok(Json(0))
}

/// Save the current genome to a file with optional ID and title parameters.
#[utoipa::path(
    post,
    path = "/v1/genome/save",
    tag = "genome",
    responses(
        (status = 200, description = "Genome saved", body = HashMap<String, String>)
    )
)]
pub async fn post_save(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    use std::fs;
    use std::path::Path;

    info!("Saving genome to file");

    // Get parameters
    let genome_id = request.get("genome_id").cloned();
    let genome_title = request.get("genome_title").cloned();
    let file_path = request.get("file_path").cloned();

    // Create save parameters
    let params = feagi_services::SaveGenomeParams {
        genome_id,
        genome_title,
    };

    // Call genome service to generate JSON
    let genome_service = state.genome_service.as_ref();
    let genome_json = genome_service
        .save_genome(params)
        .await
        .map_err(|e| ApiError::internal(format!("Failed to save genome: {}", e)))?;

    // The stored genome physiology is the artifact. The live runner period is not
    // written over simulation_timestep; a connectome restore applies the stored value.
    let genome_value: serde_json::Value = serde_json::from_str(&genome_json)
        .map_err(|e| ApiError::internal(format!("Failed to parse genome JSON: {}", e)))?;
    let genome_artifact = encode_genome_artifact(&genome_value)
        .map_err(|e| ApiError::internal(format!("Failed to encode genome artifact: {}", e)))?;

    // Determine file path
    let save_path = if let Some(path) = file_path {
        validate_genome_artifact_file_name(
            std::path::Path::new(&path)
                .file_name()
                .and_then(|value| value.to_str()),
        )?;
        std::path::PathBuf::from(path)
    } else {
        // Default: under configured data root (not cwd) so containers/read-only roots work.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        state
            .filesystem_data_root
            .join("cache")
            .join(".genome")
            .join(format!("saved_genome_{}.genome", timestamp))
    };

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&save_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ApiError::internal(format!("Failed to create directory: {}", e)))?;
    }

    // Write to file
    fs::write(&save_path, genome_artifact)
        .map_err(|e| ApiError::internal(format!("Failed to write file: {}", e)))?;

    info!("✅ Genome saved successfully to: {}", save_path.display());

    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            "Genome saved successfully".to_string(),
        ),
        ("file_path".to_string(), save_path.display().to_string()),
    ])))
}

/// Load a genome from a file by name.
#[utoipa::path(
    post,
    path = "/v1/genome/load",
    tag = "genome",
    responses(
        (status = 200, description = "Genome loaded", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_load(
    State(state): State<ApiState>,
    Json(request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let genome_name = request
        .get("genome_name")
        .ok_or_else(|| ApiError::invalid_input("genome_name required"))?;

    // Load genome from defaults
    let params = feagi_services::LoadGenomeParams {
        json_str: format!("{{\"genome_title\": \"{}\"}}", genome_name),
    };

    let genome_info = load_genome_with_priority(&state, params, "post_load").await?;

    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome loaded successfully"),
    );
    response.insert(
        "genome_title".to_string(),
        serde_json::json!(genome_info.genome_title),
    );

    Ok(Json(response))
}

/// Upload and load a genome from JSON payload.
#[utoipa::path(
    post,
    path = "/v1/genome/upload",
    tag = "genome",
    responses(
        (status = 200, description = "Genome uploaded", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_upload(
    State(state): State<ApiState>,
    Json(genome_json): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // Convert to JSON string
    let json_str = serde_json::to_string(&genome_json)
        .map_err(|e| ApiError::invalid_input(format!("Invalid JSON: {}", e)))?;

    let params = LoadGenomeParams { json_str };
    let genome_info = load_genome_with_priority(&state, params, "post_upload").await?;

    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome uploaded successfully"),
    );
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::json!(genome_info.cortical_area_count),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::json!(genome_info.brain_region_count),
    );

    Ok(Json(response))
}

/// Download the current genome as an encoded `.genome` artifact.
#[utoipa::path(
    get,
    path = "/v1/genome/download",
    tag = "genome",
    responses(
        (
            status = 200,
            description = "Genome artifact",
            body = HashMap<String, serde_json::Value>,
            content_type = "application/vnd.feagi.genome+json"
        )
    )
)]
pub async fn get_download(State(state): State<ApiState>) -> ApiResult<(HeaderMap, Vec<u8>)> {
    info!("🦀 [API] GET /v1/genome/download - Downloading current genome");
    let genome_service = state.genome_service.as_ref();

    // Get genome as JSON string
    let genome_json_str = genome_service
        .save_genome(feagi_services::types::SaveGenomeParams {
            genome_id: None,
            genome_title: None,
        })
        .await
        .map_err(|e| {
            tracing::error!("Failed to export genome: {}", e);
            ApiError::internal(format!("Failed to export genome: {}", e))
        })?;

    // Parse to Value for JSON response
    let genome_value: serde_json::Value = serde_json::from_str(&genome_json_str)
        .map_err(|e| ApiError::internal(format!("Failed to parse genome JSON: {}", e)))?;

    let genome_artifact = encode_genome_artifact(&genome_value)
        .map_err(|e| ApiError::internal(format!("Failed to encode genome artifact: {}", e)))?;

    info!(
        "✅ Genome download complete, {} bytes",
        genome_artifact.len()
    );
    Ok((
        genome_artifact_download_headers("attachment; filename=\"feagi_genome.genome\""),
        genome_artifact,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both lifecycle tests write the process-wide genome state.
    static LIFECYCLE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    #[cfg(feature = "http")]
    #[tokio::test]
    async fn connectome_transition_reports_loaded_then_error_on_failure() {
        use feagi_state_manager::{GenomeState, StateManager};
        let _lock = LIFECYCLE_TEST_LOCK.lock().await;
        let state = || StateManager::instance().read().get_genome_state();

        StateManager::instance()
            .read()
            .set_genome_state(GenomeState::Missing);
        let loaded: Result<(), ()> = with_genome_state_lifecycle(async {
            assert_eq!(
                state(),
                GenomeState::Loading,
                "loading while the import runs"
            );
            Ok(())
        })
        .await;
        assert!(loaded.is_ok());
        assert_eq!(
            state(),
            GenomeState::Loaded,
            "a restored connectome is a loaded genome"
        );

        let failed: Result<(), &str> =
            with_genome_state_lifecycle(async { Err("bad artifact") }).await;
        assert!(failed.is_err());
        assert_eq!(state(), GenomeState::Error);
    }

    #[cfg(feature = "http")]
    #[tokio::test]
    async fn rolled_back_transition_restores_previous_state_and_returns_the_original_error() {
        use feagi_services::ServiceError;
        use feagi_state_manager::{GenomeState, StateManager};
        let _lock = LIFECYCLE_TEST_LOCK.lock().await;
        let state = || StateManager::instance().read().get_genome_state();

        StateManager::instance()
            .read()
            .set_genome_state(GenomeState::Loaded);
        let rolled_back: Result<(), ServiceError> =
            with_rollback_aware_genome_state_lifecycle(async {
                assert_eq!(state(), GenomeState::Loading);
                Err(ServiceError::RolledBack(Box::new(ServiceError::Backend(
                    "rebind failed".to_string(),
                ))))
            })
            .await;
        match rolled_back {
            Err(ServiceError::Backend(message)) => assert_eq!(message, "rebind failed"),
            other => panic!("the original error must come back, got {other:?}"),
        }
        assert_eq!(state(), GenomeState::Loaded, "the previous brain is live");

        let failed: Result<(), ServiceError> = with_rollback_aware_genome_state_lifecycle(async {
            Err(ServiceError::Backend("no rollback".to_string()))
        })
        .await;
        assert!(failed.is_err());
        assert_eq!(
            state(),
            GenomeState::Error,
            "an unrestored failure is an Error"
        );
    }

    #[test]
    fn genome_artifact_file_name_requires_genome_extension() {
        assert!(validate_genome_artifact_file_name(Some("brain.genome")).is_ok());
        assert!(validate_genome_artifact_file_name(Some("brain.GENOME")).is_ok());
        assert!(validate_genome_artifact_file_name(Some(r"C:\brains\brain.genome")).is_ok());
        assert!(validate_genome_artifact_file_name(Some("brain.json")).is_err());
        assert!(validate_genome_artifact_file_name(Some(".genome")).is_err());
        assert!(validate_genome_artifact_file_name(None).is_err());
    }

    #[test]
    fn genome_download_headers_use_artifact_contract() {
        let headers =
            genome_artifact_download_headers("attachment; filename=\"feagi_genome.genome\"");
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            GENOME_ARTIFACT_MEDIA_TYPE
        );
        assert_eq!(
            headers.get(header::CONTENT_DISPOSITION).unwrap(),
            "attachment; filename=\"feagi_genome.genome\""
        );
    }
}

/// Get genome properties including metadata, size, and configuration details.
#[utoipa::path(
    get,
    path = "/v1/genome/properties",
    tag = "genome",
    responses(
        (status = 200, description = "Genome properties", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_properties(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement proper metadata retrieval from genome service
    Ok(Json(HashMap::new()))
}

/// Validate a genome structure for correctness and completeness.
#[utoipa::path(
    post,
    path = "/v1/genome/validate",
    tag = "genome",
    responses(
        (status = 200, description = "Validation result", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_validate(
    State(_state): State<ApiState>,
    Json(_genome): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome validation
    let mut response = HashMap::new();
    response.insert("valid".to_string(), serde_json::json!(true));
    response.insert("errors".to_string(), serde_json::json!([]));
    response.insert("warnings".to_string(), serde_json::json!([]));

    Ok(Json(response))
}

/// Transform genome between different formats (flat to hierarchical or vice versa).
#[utoipa::path(
    post,
    path = "/v1/genome/transform",
    tag = "genome",
    responses(
        (status = 200, description = "Transformed genome", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_transform(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome transformation
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome transformation not yet implemented"),
    );

    Ok(Json(response))
}

/// Clone the current genome with a new name, creating an independent copy.
#[utoipa::path(
    post,
    path = "/v1/genome/clone",
    tag = "genome",
    responses(
        (status = 200, description = "Genome cloned", body = HashMap<String, String>)
    )
)]
pub async fn post_clone(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // TODO: Implement genome cloning
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Genome cloning not yet implemented".to_string(),
    )])))
}

/// Reset genome to its default state, clearing all cortical areas and brain regions.
/// Use before loading a new genome when "cortical area already exists" errors occur.
#[utoipa::path(
    post,
    path = "/v1/genome/reset",
    tag = "genome",
    responses(
        (status = 200, description = "Genome reset", body = HashMap<String, String>),
        (status = 409, description = "Genome transition in progress"),
        (status = 500, description = "Reset failed")
    )
)]
pub async fn post_reset(State(state): State<ApiState>) -> ApiResult<Json<HashMap<String, String>>> {
    let _lock = state.genome_transition_lock.try_write().map_err(|_| {
        ApiError::conflict("Another genome transition is in progress; wait for it to finish")
    })?;

    let genome_service = state.genome_service.as_ref();
    genome_service.reset_connectome().await.map_err(|e| {
        tracing::error!(target: "feagi-api", "Genome reset failed: {}", e);
        ApiError::internal(format!("Genome reset failed: {}", e))
    })?;

    info!(target: "feagi-api", "Genome reset complete - connectome cleared");
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Genome reset complete. Connectome cleared. Load a new genome to continue.".to_string(),
    )])))
}

/// Get genome metadata (alternative endpoint to properties).
#[utoipa::path(
    get,
    path = "/v1/genome/metadata",
    tag = "genome",
    responses(
        (status = 200, description = "Genome metadata", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_metadata(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    get_properties(State(state)).await
}

/// Merge another genome into the current genome, combining their structures.
#[utoipa::path(
    post,
    path = "/v1/genome/merge",
    tag = "genome",
    responses(
        (status = 200, description = "Genome merged", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_merge(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, serde_json::Value>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome merging
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome merging not yet implemented"),
    );

    Ok(Json(response))
}

/// Get a diff comparison between two genomes showing their differences.
#[utoipa::path(
    get,
    path = "/v1/genome/diff",
    tag = "genome",
    params(
        ("genome_a" = String, Query, description = "First genome name"),
        ("genome_b" = String, Query, description = "Second genome name")
    ),
    responses(
        (status = 200, description = "Genome diff", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_diff(
    State(_state): State<ApiState>,
    Query(_params): Query<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement genome diffing
    let mut response = HashMap::new();
    response.insert("differences".to_string(), serde_json::json!([]));

    Ok(Json(response))
}

/// Export genome in a specific format (JSON, YAML, binary, etc.).
#[utoipa::path(
    post,
    path = "/v1/genome/export_format",
    tag = "genome",
    responses(
        (status = 200, description = "Exported genome", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn post_export_format(
    State(_state): State<ApiState>,
    Json(_request): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    // TODO: Implement format-specific export
    let mut response = HashMap::new();
    response.insert(
        "message".to_string(),
        serde_json::json!("Format export not yet implemented"),
    );

    Ok(Json(response))
}

// EXACT Python paths:
/// Get current amalgamation status and configuration.
#[utoipa::path(get, path = "/v1/genome/amalgamation", tag = "genome")]
pub async fn get_amalgamation(
    State(state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let lock = state.amalgamation_state.read();
    let mut response = HashMap::new();
    if let Some(p) = lock.pending.as_ref() {
        response.insert(
            "pending".to_string(),
            amalgamation::pending_summary_to_health_json(&p.summary),
        );
    } else {
        response.insert("pending".to_string(), serde_json::Value::Null);
    }
    Ok(Json(response))
}

/// Get history of all genome amalgamation operations performed.
#[utoipa::path(get, path = "/v1/genome/amalgamation_history", tag = "genome")]
pub async fn get_amalgamation_history_exact(
    State(state): State<ApiState>,
) -> ApiResult<Json<Vec<HashMap<String, serde_json::Value>>>> {
    let lock = state.amalgamation_state.read();
    let mut out: Vec<HashMap<String, serde_json::Value>> = Vec::new();
    for entry in &lock.history {
        out.push(HashMap::from([
            (
                "amalgamation_id".to_string(),
                serde_json::json!(entry.amalgamation_id),
            ),
            (
                "genome_title".to_string(),
                serde_json::json!(entry.genome_title),
            ),
            (
                "circuit_size".to_string(),
                serde_json::json!(entry.circuit_size),
            ),
            ("status".to_string(), serde_json::json!(entry.status)),
            (
                "timestamp_ms".to_string(),
                serde_json::json!(entry.timestamp_ms),
            ),
        ]));
    }
    Ok(Json(out))
}

/// Get metadata about all available cortical types including supported encodings and configurations.
#[utoipa::path(get, path = "/v1/genome/cortical_template", tag = "genome")]
pub async fn get_cortical_template(
    State(_state): State<ApiState>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
        FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
    };
    use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
    use serde_json::json;

    let mut templates = HashMap::new();

    // Helper to convert data type to human-readable format.
    //
    // NOTE: This endpoint is designed for tool/UIs (e.g. BV) and must be
    // deterministic across platforms and runs. No fallbacks.
    let data_type_to_json = |dt: IOCorticalAreaConfigurationFlag| -> serde_json::Value {
        let (variant, frame, positioning) = match dt {
            IOCorticalAreaConfigurationFlag::Boolean => {
                ("Boolean", FrameChangeHandling::Absolute, None)
            }
            IOCorticalAreaConfigurationFlag::Percentage(f, p) => ("Percentage", f, Some(p)),
            IOCorticalAreaConfigurationFlag::Percentage2D(f, p) => ("Percentage2D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::Percentage3D(f, p) => ("Percentage3D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::Percentage4D(f, p) => ("Percentage4D", f, Some(p)),
            IOCorticalAreaConfigurationFlag::SignedPercentage(f, p) => {
                ("SignedPercentage", f, Some(p))
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage2D(f, p) => {
                ("SignedPercentage2D", f, Some(p))
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage3D(f, p) => {
                ("SignedPercentage3D", f, Some(p))
            }
            IOCorticalAreaConfigurationFlag::SignedPercentage4D(f, p) => {
                ("SignedPercentage4D", f, Some(p))
            }
            IOCorticalAreaConfigurationFlag::CartesianPlane(f) => ("CartesianPlane", f, None),
            IOCorticalAreaConfigurationFlag::Misc(f) => ("Misc", f, None),
            IOCorticalAreaConfigurationFlag::PoseEstimation(f, _) => ("PoseEstimation", f, None),
        };

        let frame_str = match frame {
            FrameChangeHandling::Absolute => "Absolute",
            FrameChangeHandling::Incremental => "Incremental",
        };

        let positioning_str = positioning.map(|p| match p {
            PercentageNeuronPositioning::Linear => "Linear",
            PercentageNeuronPositioning::Fractional => "Fractional",
        });

        json!({
            "variant": variant,
            "frame_change_handling": frame_str,
            "percentage_positioning": positioning_str,
            "config_value": dt.to_data_type_configuration_flag()
        })
    };

    // Add motor types
    for motor_unit in MotorCorticalUnit::list_all() {
        let friendly_name = motor_unit.get_friendly_name();
        let cortical_id_ref = motor_unit.get_cortical_id_unit_reference();
        let num_areas = motor_unit.get_number_cortical_areas();
        let topology = motor_unit.get_unit_default_topology();

        // BREAKING CHANGE (unreleased API):
        // - Remove unit-level `supported_data_types`.
        // - Expose per-subunit metadata, because some units (e.g. Gaze) have heterogeneous subunits
        //   with different IOCorticalAreaConfigurationFlag variants (Percentage2D vs Percentage).
        //
        // We derive supported types by:
        // - generating canonical cortical IDs from the MotorCorticalUnit template for each
        //   (frame_change_handling, percentage_neuron_positioning) combination
        // - extracting the IO configuration flag from each cortical ID
        // - grouping supported_data_types per subunit index
        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use serde_json::{Map, Value};
        use std::collections::HashMap as StdHashMap;

        let mut subunits: StdHashMap<String, serde_json::Value> = StdHashMap::new();

        // Initialize subunits with topology-derived properties.
        for (sub_idx, topo) in topology {
            subunits.insert(
                sub_idx.get().to_string(),
                json!({
                    "relative_position": topo.relative_position,
                    "channel_dimensions_default": topo.channel_dimensions_default,
                    "channel_dimensions_min": topo.channel_dimensions_min,
                    "channel_dimensions_max": topo.channel_dimensions_max,
                    "supported_data_types": Vec::<serde_json::Value>::new(),
                }),
            );
        }

        // Build per-subunit supported_data_types (deduped).
        let allowed_frames = motor_unit.get_allowed_frame_change_handling();
        let frames: Vec<FrameChangeHandling> = match allowed_frames {
            Some(allowed) => allowed.to_vec(),
            None => vec![
                FrameChangeHandling::Absolute,
                FrameChangeHandling::Incremental,
            ],
        };

        let positionings = [
            PercentageNeuronPositioning::Linear,
            PercentageNeuronPositioning::Fractional,
        ];

        let mut per_subunit_dedup: StdHashMap<String, std::collections::HashSet<String>> =
            StdHashMap::new();

        for frame in frames {
            for positioning in positionings {
                let mut map: Map<String, Value> = Map::new();
                map.insert(
                    "frame_change_handling".to_string(),
                    serde_json::to_value(frame).unwrap_or(Value::Null),
                );
                map.insert(
                    "percentage_neuron_positioning".to_string(),
                    serde_json::to_value(positioning).unwrap_or(Value::Null),
                );

                // Use unit index 0 for template enumeration (index does not affect IO flags).
                let cortical_ids = motor_unit
                    .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                        CorticalUnitIndex::from(0u16),
                        map,
                    );

                if let Ok(ids) = cortical_ids {
                    for (i, id) in ids.into_iter().enumerate() {
                        if let Ok(flag) = id.extract_io_data_flag() {
                            let dt_json = data_type_to_json(flag);
                            let subunit_key = i.to_string();

                            let dedup_key = format!(
                                "{}|{}|{}",
                                dt_json
                                    .get("variant")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(""),
                                dt_json
                                    .get("frame_change_handling")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(""),
                                dt_json
                                    .get("percentage_positioning")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                            );

                            let seen = per_subunit_dedup.entry(subunit_key.clone()).or_default();
                            if !seen.insert(dedup_key) {
                                continue;
                            }

                            if let Some(subunit_obj) = subunits.get_mut(&subunit_key) {
                                if let Some(arr) = subunit_obj
                                    .get_mut("supported_data_types")
                                    .and_then(|v| v.as_array_mut())
                                {
                                    arr.push(dt_json);
                                }
                            }
                        }
                    }
                }
            }
        }

        templates.insert(
            format!("o{}", String::from_utf8_lossy(&cortical_id_ref)),
            json!({
                "type": "motor",
                "friendly_name": friendly_name,
                "cortical_id_prefix": String::from_utf8_lossy(&cortical_id_ref).to_string(),
                "number_of_cortical_areas": num_areas,
                "subunits": subunits,
                "description": format!("Motor output: {}", friendly_name)
            }),
        );
    }

    // Add sensory types
    for sensory_unit in SensoryCorticalUnit::list_all() {
        let friendly_name = sensory_unit.get_friendly_name();
        let cortical_id_ref = sensory_unit.get_cortical_id_unit_reference();
        let num_areas = sensory_unit.get_number_cortical_areas();
        let topology = sensory_unit.get_unit_default_topology();

        use feagi_structures::genomic::cortical_area::descriptors::CorticalUnitIndex;
        use serde_json::{Map, Value};
        use std::collections::HashMap as StdHashMap;

        let mut subunits: StdHashMap<String, serde_json::Value> = StdHashMap::new();

        for (sub_idx, topo) in topology {
            subunits.insert(
                sub_idx.get().to_string(),
                json!({
                    "relative_position": topo.relative_position,
                    "channel_dimensions_default": topo.channel_dimensions_default,
                    "channel_dimensions_min": topo.channel_dimensions_min,
                    "channel_dimensions_max": topo.channel_dimensions_max,
                    "supported_data_types": Vec::<serde_json::Value>::new(),
                }),
            );
        }

        let allowed_frames = sensory_unit.get_allowed_frame_change_handling();
        let frames: Vec<FrameChangeHandling> = match allowed_frames {
            Some(allowed) => allowed.to_vec(),
            None => vec![
                FrameChangeHandling::Absolute,
                FrameChangeHandling::Incremental,
            ],
        };

        let positionings = [
            PercentageNeuronPositioning::Linear,
            PercentageNeuronPositioning::Fractional,
        ];

        let mut per_subunit_dedup: StdHashMap<String, std::collections::HashSet<String>> =
            StdHashMap::new();

        for frame in frames {
            for positioning in positionings {
                let mut map: Map<String, Value> = Map::new();
                map.insert(
                    "frame_change_handling".to_string(),
                    serde_json::to_value(frame).unwrap_or(Value::Null),
                );
                map.insert(
                    "percentage_neuron_positioning".to_string(),
                    serde_json::to_value(positioning).unwrap_or(Value::Null),
                );

                let cortical_ids = sensory_unit
                    .get_cortical_id_vector_from_index_and_serde_io_configuration_flags(
                        CorticalUnitIndex::from(0u16),
                        map,
                    );

                if let Ok(ids) = cortical_ids {
                    for (i, id) in ids.into_iter().enumerate() {
                        if let Ok(flag) = id.extract_io_data_flag() {
                            let dt_json = data_type_to_json(flag);
                            let subunit_key = i.to_string();

                            let dedup_key = format!(
                                "{}|{}|{}",
                                dt_json
                                    .get("variant")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(""),
                                dt_json
                                    .get("frame_change_handling")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or(""),
                                dt_json
                                    .get("percentage_positioning")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                            );

                            let seen = per_subunit_dedup.entry(subunit_key.clone()).or_default();
                            if !seen.insert(dedup_key) {
                                continue;
                            }

                            if let Some(subunit_obj) = subunits.get_mut(&subunit_key) {
                                if let Some(arr) = subunit_obj
                                    .get_mut("supported_data_types")
                                    .and_then(|v| v.as_array_mut())
                                {
                                    arr.push(dt_json);
                                }
                            }
                        }
                    }
                }
            }
        }

        templates.insert(
            format!("i{}", String::from_utf8_lossy(&cortical_id_ref)),
            json!({
                "type": "sensory",
                "friendly_name": friendly_name,
                "cortical_id_prefix": String::from_utf8_lossy(&cortical_id_ref).to_string(),
                "number_of_cortical_areas": num_areas,
                "subunits": subunits,
                "description": format!("Sensory input: {}", friendly_name)
            }),
        );
    }

    Ok(Json(templates))
}

/// Get list of available embedded default genome templates (barebones, essential, test, vision).
#[utoipa::path(get, path = "/v1/genome/defaults/files", tag = "genome")]
pub async fn get_defaults_files(State(_state): State<ApiState>) -> ApiResult<Json<Vec<String>>> {
    Ok(Json(vec![
        "barebones".to_string(),
        "essential".to_string(),
        "test".to_string(),
        "vision".to_string(),
    ]))
}

/// Download a specific brain region from the genome.
#[utoipa::path(
    get,
    path = "/v1/genome/download_region",
    tag = "genome",
    responses(
        (
            status = 200,
            description = "Region genome artifact",
            body = HashMap<String, serde_json::Value>,
            content_type = "application/vnd.feagi.genome+json"
        )
    )
)]
pub async fn get_download_region(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<(HeaderMap, Vec<u8>)> {
    let region_id = params
        .get("region_id")
        .cloned()
        .ok_or_else(|| ApiError::invalid_input("region_id query parameter is required"))?;
    let json_str = state
        .genome_service
        .export_region_genome(region_id)
        .await
        .map_err(ApiError::from)?;
    let value: serde_json::Value = serde_json::from_str(&json_str).map_err(|e| {
        ApiError::internal(format!("Exported region genome JSON is invalid: {}", e))
    })?;
    let artifact = encode_genome_artifact(&value)
        .map_err(|e| ApiError::internal(format!("Failed to encode region artifact: {}", e)))?;
    Ok((
        genome_artifact_download_headers("attachment; filename=\"feagi_region.genome\""),
        artifact,
    ))
}

/// Get the current genome number or generation identifier.
#[utoipa::path(get, path = "/v1/genome/genome_number", tag = "genome")]
pub async fn get_genome_number(State(_state): State<ApiState>) -> ApiResult<Json<i32>> {
    Ok(Json(0))
}

/// Perform genome amalgamation by specifying a filename.
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_filename", tag = "genome")]
pub async fn post_amalgamation_by_filename(
    State(state): State<ApiState>,
    Json(req): Json<HashMap<String, String>>,
) -> ApiResult<Json<HashMap<String, String>>> {
    // Deterministic implementation:
    // - Supports embedded Rust template genomes by name (no filesystem I/O).
    // - For all other filenames, require /amalgamation_by_payload.
    let file_name = req
        .get("file_name")
        .or_else(|| req.get("filename"))
        .or_else(|| req.get("genome_file_name"))
        .ok_or_else(|| ApiError::invalid_input("file_name required"))?;

    let genome_json = match file_name.as_str() {
        "barebones" => feagi_evolutionary::BAREBONES_GENOME_JSON.to_string(),
        "essential" => feagi_evolutionary::ESSENTIAL_GENOME_JSON.to_string(),
        "test" => feagi_evolutionary::TEST_GENOME_JSON.to_string(),
        "vision" => feagi_evolutionary::VISION_GENOME_JSON.to_string(),
        other => {
            return Err(ApiError::invalid_input(format!(
                "Unsupported file_name '{}'. Use /v1/genome/amalgamation_by_payload for arbitrary genomes.",
                other
            )))
        }
    };

    let amalgamation_id =
        queue_amalgamation_from_genome_json_str(&state, genome_json, None, None, None).await?;

    Ok(Json(HashMap::from([
        ("message".to_string(), "Amalgamation queued".to_string()),
        ("amalgamation_id".to_string(), amalgamation_id),
    ])))
}

/// Perform genome amalgamation using a direct JSON payload.
#[utoipa::path(post, path = "/v1/genome/amalgamation_by_payload", tag = "genome")]
pub async fn post_amalgamation_by_payload(
    State(state): State<ApiState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let json_str = serde_json::to_string(&req)
        .map_err(|e| ApiError::invalid_input(format!("Invalid JSON: {}", e)))?;
    let amalgamation_id =
        queue_amalgamation_from_genome_json_str(&state, json_str, None, None, None).await?;

    Ok(Json(HashMap::from([
        ("message".to_string(), "Amalgamation queued".to_string()),
        ("amalgamation_id".to_string(), amalgamation_id),
    ])))
}

/// Queue a genome payload together with a fit plan (the dimensions to create its areas at).
///
/// Body: `{ "genome": <full genome document>, "fit_plan": { "areas": [...], "adapter": {...},
/// "morphologies": [...], "mappings": [...] } }` (all plan fields optional). Behaves as
/// `/v1/genome/amalgamation_by_payload` (which keeps the whole body as the genome), plus the plan
/// is validated now (including the host areas its mappings end on) and applied when the
/// amalgamation is confirmed: areas at their fitted size, then the adapter region and areas, the
/// morphologies and the mappings, all or nothing. An invalid plan returns 400 and queues nothing.
/// A plan with no guest areas (an empty genome) wires existing host areas later: the same path.
#[utoipa::path(
    post,
    path = "/v1/genome/amalgamation_by_payload_with_fit",
    tag = "genome"
)]
pub async fn post_amalgamation_by_payload_with_fit(
    State(state): State<ApiState>,
    Json(req): Json<serde_json::Value>,
) -> ApiResult<Json<HashMap<String, String>>> {
    let genome = req
        .get("genome")
        .filter(|value| !value.is_null())
        .ok_or_else(|| ApiError::invalid_input("Body must contain a 'genome' object"))?;
    let fit_plan = match req.get("fit_plan") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => Some(
            serde_json::from_value::<feagi_evolutionary::FitPlan>(value.clone())
                .map_err(|e| ApiError::invalid_input(format!("Invalid fit_plan: {}", e)))?,
        ),
    };
    let source_release_id = req
        .get("source_release_id")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let source_catalog = match req.get("source_catalog") {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            let catalog = value.as_str().map(str::trim).unwrap_or_default();
            if catalog.is_empty() || catalog.len() > 128 {
                return Err(ApiError::invalid_input(
                    "source_catalog must be a non-empty string of at most 128 characters",
                ));
            }
            Some(catalog.to_string())
        }
    };
    let json_str = serde_json::to_string(genome)
        .map_err(|e| ApiError::invalid_input(format!("Invalid JSON: {}", e)))?;
    let amalgamation_id = queue_amalgamation_from_genome_json_str(
        &state,
        json_str,
        fit_plan,
        source_release_id,
        source_catalog,
    )
    .await?;

    Ok(Json(HashMap::from([
        ("message".to_string(), "Amalgamation queued".to_string()),
        ("amalgamation_id".to_string(), amalgamation_id),
    ])))
}

/// Perform genome amalgamation by uploading a genome file.
#[cfg(feature = "http")]
#[utoipa::path(
    post,
    path = "/v1/genome/amalgamation_by_upload",
    tag = "genome",
    request_body(content = GenomeFileUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Amalgamation queued", body = HashMap<String, String>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_amalgamation_by_upload(
    State(state): State<ApiState>,
    mut multipart: Multipart,
) -> ApiResult<Json<HashMap<String, String>>> {
    let mut genome_json: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::invalid_input(format!("Invalid multipart upload: {}", e)))?
    {
        if field.name() == Some("file") {
            validate_genome_artifact_file_name(field.file_name())?;
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::invalid_input(format!("Failed to read uploaded file: {}", e))
            })?;

            genome_json = Some(decode_genome_artifact_for_json_service(&bytes)?);
            break;
        }
    }

    let json_str =
        genome_json.ok_or_else(|| ApiError::invalid_input("Missing multipart field 'file'"))?;
    let amalgamation_id =
        queue_amalgamation_from_genome_json_str(&state, json_str, None, None, None).await?;

    Ok(Json(HashMap::from([
        ("message".to_string(), "Amalgamation queued".to_string()),
        ("amalgamation_id".to_string(), amalgamation_id),
    ])))
}

/// Append structures to the genome from a file.
#[cfg(feature = "http")]
#[utoipa::path(
    post,
    path = "/v1/genome/append-file",
    tag = "genome",
    request_body(content = GenomeFileUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Append processed", body = HashMap<String, String>)
    )
)]
pub async fn post_append_file(
    State(_state): State<ApiState>,
    mut _multipart: Multipart,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Upload and load a genome from a file.
#[cfg(feature = "http")]
#[utoipa::path(
    post,
    path = "/v1/genome/upload/file",
    tag = "genome",
    request_body(content = GenomeFileUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Genome uploaded", body = HashMap<String, serde_json::Value>),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_upload_file(
    State(state): State<ApiState>,
    mut multipart: Multipart,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let mut genome_json: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::invalid_input(format!("Invalid multipart upload: {}", e)))?
    {
        if field.name() == Some("file") {
            validate_genome_artifact_file_name(field.file_name())?;
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::invalid_input(format!("Failed to read uploaded file: {}", e))
            })?;

            genome_json = Some(decode_genome_artifact_for_json_service(&bytes)?);
            break;
        }
    }

    let json_str =
        genome_json.ok_or_else(|| ApiError::invalid_input("Missing multipart field 'file'"))?;

    let genome_info =
        load_genome_with_priority(&state, LoadGenomeParams { json_str }, "post_upload_file")
            .await?;

    let mut response = HashMap::new();
    response.insert("success".to_string(), serde_json::json!(true));
    response.insert(
        "message".to_string(),
        serde_json::json!("Genome uploaded successfully"),
    );
    response.insert(
        "cortical_area_count".to_string(),
        serde_json::json!(genome_info.cortical_area_count),
    );
    response.insert(
        "brain_region_count".to_string(),
        serde_json::json!(genome_info.brain_region_count),
    );

    Ok(Json(response))
}

/// Upload a genome file with edit mode enabled.
#[cfg(feature = "http")]
#[utoipa::path(
    post,
    path = "/v1/genome/upload/file/edit",
    tag = "genome",
    request_body(content = GenomeFileUploadForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Upload processed", body = HashMap<String, String>)
    )
)]
pub async fn post_upload_file_edit(
    State(_state): State<ApiState>,
    mut _multipart: Multipart,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}

/// Upload and load a genome from a JSON string.
#[utoipa::path(post, path = "/v1/genome/upload/string", tag = "genome")]
pub async fn post_upload_string(
    State(_state): State<ApiState>,
    Json(_req): Json<String>,
) -> ApiResult<Json<HashMap<String, String>>> {
    Ok(Json(HashMap::from([(
        "message".to_string(),
        "Not yet implemented".to_string(),
    )])))
}
