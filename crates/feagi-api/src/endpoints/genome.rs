// Copyright 2025 Neuraville Inc.
// Licensed under the Apache License, Version 2.0

//! Genome API Endpoints - Exact port from Python `/v1/genome/*`

// Removed - using crate::common::State instead
use crate::amalgamation;
use crate::common::ApiState;
use crate::common::{ApiError, ApiResult, Json, Query, State};
use feagi_services::types::{GenomeInfo, LoadGenomeParams};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

#[cfg(feature = "http")]
use axum::extract::Multipart;

/// Multipart file upload schema for Swagger UI.
///
/// This enables Swagger to show a file picker for endpoints that accept genome JSON files.
#[derive(Debug, Clone, utoipa::ToSchema)]
pub struct GenomeFileUploadForm {
    /// Genome JSON file contents.
    #[schema(value_type = String, format = Binary)]
    pub file: String,
}

fn queue_amalgamation_from_genome_json_str(
    state: &ApiState,
    genome_json: String,
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

    let genome = feagi_evolutionary::load_genome_from_json(&genome_json)
        .map_err(|e| ApiError::invalid_input(format!("Invalid genome payload: {}", e)))?;

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
        genome_json,
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
    let _transition_lock = state.genome_transition_lock.try_lock().map_err(|_| {
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
                .iter()
                .filter_map(|(sid, _)| guard.get_device_registrations_by_agent(*sid).cloned())
                .collect()
        };
        for device_regs in device_regs_list {
            crate::common::agent_registration::auto_create_cortical_areas_from_device_registrations(
                state,
                &device_regs,
            )
            .await;
        }
    }

    Ok(genome_info)
}

/// Inject the current runtime simulation timestep (seconds) into a genome JSON value.
///
/// Rationale: the burst engine timestep can be updated at runtime, but `GenomeService::save_genome()`
/// serializes the stored `RuntimeGenome` (which may still have the older physiology value).
/// This keeps exported/saved genomes consistent with the *current* FEAGI simulation state.
fn inject_simulation_timestep_into_genome(
    mut genome: serde_json::Value,
    simulation_timestep_s: f64,
) -> Result<serde_json::Value, ApiError> {
    let physiology = genome
        .get_mut("physiology")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| {
            ApiError::internal(
                "Genome JSON missing required object key 'physiology' while saving".to_string(),
            )
        })?;

    physiology.insert(
        "simulation_timestep".to_string(),
        serde_json::Value::from(simulation_timestep_s),
    );
    Ok(genome)
}

async fn get_current_runtime_simulation_timestep_s(state: &ApiState) -> Result<f64, ApiError> {
    let runtime_service = state.runtime_service.as_ref();
    let status = runtime_service
        .get_status()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get runtime status: {}", e)))?;

    // Convert frequency (Hz) to timestep (seconds).
    Ok(if status.frequency_hz > 0.0 {
        1.0 / status.frequency_hz
    } else {
        0.0
    })
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

    // Resolve and consume the pending request.
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

    // 1) Create a new brain region to host the imported circuit.
    // Note: ConnectomeServiceImpl shares the same RuntimeGenome Arc with GenomeServiceImpl, so
    // persisting the region into the RuntimeGenome is required for subsequent cortical-area creation.
    let connectome_service = state.connectome_service.as_ref();

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

    let host_cortical_ids: std::collections::HashSet<String> = connectome_service
        .get_cortical_area_ids()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to list cortical area IDs: {}", e)))?
        .into_iter()
        .collect();

    let remapped_guest_custom_memory_ids =
        feagi_evolutionary::remap_guest_custom_memory_cortical_ids_for_amalgamation(
            &mut imported_genome,
            &host_cortical_ids,
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
    let mut to_create: Vec<feagi_services::types::CreateCorticalAreaParams> = Vec::new();
    let mut skipped_existing: Vec<String> = Vec::new();

    // Get root region ID for IPU/OPU areas
    let root_region_id = connectome_service
        .get_root_region_id()
        .await
        .map_err(|e| ApiError::internal(format!("Failed to get root region ID: {}", e)))?;

    for area in imported_genome.cortical_areas.values() {
        let cortical_id = area.cortical_id.as_base_64();
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
            skipped_existing.push(cortical_id);
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
                // Custom, Memory, Core areas go to amalgamation region
                amalgamation_id.clone()
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
    if !to_create.is_empty() {
        genome_service
            .create_cortical_areas(to_create)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to import cortical areas: {}", e)))?;
    }

    // 3) Import morphologies used by the imported areas' cortical mappings.
    //
    // Collect all morphology IDs referenced in the cortical_mapping_dst of imported areas,
    // then import those morphologies from the imported genome into the current genome.
    let imported_area_ids: std::collections::HashSet<String> = imported_genome
        .cortical_areas
        .keys()
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
                        "🧬 [AMALGAMATION] Imported mapping {} -> {} ({} synapses)",
                        src_area_id,
                        dst_area_id,
                        synapse_count
                    );
                    imported_mapping_count += 1;
                }
                Err(e) => {
                    tracing::warn!(
                        target: "feagi-api",
                        "🧬 [AMALGAMATION] Failed to import mapping {} -> {}: {}",
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
        if imported_morphology_count > 0 {
            state_manager
                .set_morphologies_hash(state_manager.get_morphologies_hash().wrapping_add(1));
        }
        if imported_mapping_count > 0 {
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

    Ok(Json(HashMap::from([
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
            "post_merge_brain_region_count".to_string(),
            serde_json::json!(post_merge_brain_region_count),
        ),
        (
            "post_merge_cortical_area_total".to_string(),
            serde_json::json!(post_merge_cortical_area_total),
        ),
    ])))
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

    // Ensure physiology.simulation_timestep reflects the *current* runtime timestep at save time.
    let simulation_timestep_s = get_current_runtime_simulation_timestep_s(&state).await?;
    let genome_value: serde_json::Value = serde_json::from_str(&genome_json)
        .map_err(|e| ApiError::internal(format!("Failed to parse genome JSON: {}", e)))?;
    let genome_value = inject_simulation_timestep_into_genome(genome_value, simulation_timestep_s)?;
    let genome_json = serde_json::to_string_pretty(&genome_value)
        .map_err(|e| ApiError::internal(format!("Failed to serialize genome JSON: {}", e)))?;

    // Determine file path
    let save_path = if let Some(path) = file_path {
        std::path::PathBuf::from(path)
    } else {
        // Default to hidden genome directory with timestamp.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        std::path::PathBuf::from(".genome").join(format!("saved_genome_{}.json", timestamp))
    };

    // Ensure parent directory exists
    if let Some(parent) = Path::new(&save_path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| ApiError::internal(format!("Failed to create directory: {}", e)))?;
    }

    // Write to file
    fs::write(&save_path, genome_json)
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

/// Download the current genome as a JSON document.
#[utoipa::path(
    get,
    path = "/v1/genome/download",
    tag = "genome",
    responses(
        (status = 200, description = "Genome JSON", body = HashMap<String, serde_json::Value>)
    )
)]
pub async fn get_download(State(state): State<ApiState>) -> ApiResult<Json<serde_json::Value>> {
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

    // Ensure physiology.simulation_timestep reflects the *current* runtime timestep at download time.
    let simulation_timestep_s = get_current_runtime_simulation_timestep_s(&state).await?;
    let genome_value = inject_simulation_timestep_into_genome(genome_value, simulation_timestep_s)?;

    info!(
        "✅ Genome download complete, {} bytes",
        genome_json_str.len()
    );
    Ok(Json(genome_value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_inject_simulation_timestep_into_genome_updates_physio_key() {
        let genome = json!({
            "version": "3.0",
            "physiology": {
                "simulation_timestep": 0.025,
                "max_age": 10000000
            }
        });

        let updated = inject_simulation_timestep_into_genome(genome, 0.05).unwrap();
        assert_eq!(updated["physiology"]["simulation_timestep"], json!(0.05));
        assert_eq!(updated["physiology"]["max_age"], json!(10000000));
    }

    #[test]
    fn test_inject_simulation_timestep_into_genome_errors_when_missing_physio() {
        let genome = json!({ "version": "3.0" });
        let err = inject_simulation_timestep_into_genome(genome, 0.05).unwrap_err();
        assert!(format!("{err:?}").contains("physiology"));
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
    let _lock = state.genome_transition_lock.try_lock().map_err(|_| {
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
                        CorticalUnitIndex::from(0u8),
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
                        CorticalUnitIndex::from(0u8),
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
#[utoipa::path(get, path = "/v1/genome/download_region", tag = "genome")]
pub async fn get_download_region(
    State(state): State<ApiState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
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
    Ok(Json(value))
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

    let amalgamation_id = queue_amalgamation_from_genome_json_str(&state, genome_json)?;

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
    let amalgamation_id = queue_amalgamation_from_genome_json_str(&state, json_str)?;

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
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::invalid_input(format!("Failed to read uploaded file: {}", e))
            })?;

            let json_str = std::str::from_utf8(&bytes).map_err(|e| {
                ApiError::invalid_input(format!(
                    "Uploaded file must be UTF-8 encoded JSON (decode error: {})",
                    e
                ))
            })?;
            genome_json = Some(json_str.to_string());
            break;
        }
    }

    let json_str =
        genome_json.ok_or_else(|| ApiError::invalid_input("Missing multipart field 'file'"))?;
    let amalgamation_id = queue_amalgamation_from_genome_json_str(&state, json_str)?;

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
            let bytes = field.bytes().await.map_err(|e| {
                ApiError::invalid_input(format!("Failed to read uploaded file: {}", e))
            })?;

            let json_str = std::str::from_utf8(&bytes).map_err(|e| {
                ApiError::invalid_input(format!(
                    "Uploaded file must be UTF-8 encoded JSON (decode error: {})",
                    e
                ))
            })?;
            genome_json = Some(json_str.to_string());
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
