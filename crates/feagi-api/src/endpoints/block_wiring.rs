// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Wiring an added block as its own call: one adapter region (areas, morphologies, mappings) next
to a block region that already exists in the brain.

- `POST /v1/genome/block_wiring` creates the adapter, or replaces one (`replaces_adapter_region_id`,
  "rewire": remove and create as one all-or-nothing call).
- `POST /v1/genome/block_wiring_removal` removes an adapter with everything it created.

Body-free: ids, names, dimensions, coordinates, mapping rules and an opaque `stamp` the caller
wants kept. Mapping ends are existing host areas (the block's areas included) or adapter areas.
The adapter region stores `adapter_for` (the block region id), the caller's `stamp` and
`adapter_wiring` (exactly what was created), all in the genome, so they are saved and loaded.
Removal and rewire act on that record.
*/

use crate::common::{ApiError, ApiResult, ApiState, Json, State};
use feagi_evolutionary::{AdapterArea, AdapterPlan, FitPlan, PlanMapping, PlanMorphology};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::genome::{check_plan_wiring, is_not_found, read_plan_host};

type Properties = HashMap<String, serde_json::Value>;

/// Region property: id of the block region this adapter serves.
pub const ADAPTER_FOR_KEY: &str = "adapter_for";
/// Region property: the caller's opaque stamp.
pub const STAMP_KEY: &str = "stamp";
/// Region property: what the adapter created (areas, morphologies, mappings).
pub const ADAPTER_WIRING_KEY: &str = "adapter_wiring";

#[derive(Debug, Deserialize)]
pub struct BlockWiringRequest {
    pub block_region_id: String,
    pub adapter: AdapterPlan,
    #[serde(default)]
    pub morphologies: Vec<PlanMorphology>,
    #[serde(default)]
    pub mappings: Vec<PlanMapping>,
    /// Opaque to feagi-core; stored on the adapter region as given.
    #[serde(default)]
    pub stamp: serde_json::Value,
    /// An adapter region to replace (rewire): removed and the new one created, or nothing changes.
    #[serde(default)]
    pub replaces_adapter_region_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlockWiringRemovalRequest {
    pub adapter_region_id: String,
}

/// What an adapter holds; stored on its region and used to remove or restore it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AdapterRecord {
    areas: Vec<AdapterArea>,
    morphologies: Vec<PlanMorphology>,
    mappings: Vec<PlanMapping>,
}

/// An adapter to create, with its region's final properties.
struct AdapterSpec {
    region_id: String,
    region_name: String,
    parent_id: String,
    properties: Properties,
    record: AdapterRecord,
}

#[derive(Default)]
struct Created {
    regions: Vec<String>,
    areas: Vec<String>,
    morphologies: Vec<String>,
    mappings: Vec<(String, String)>,
}

fn region_origin(properties: &Properties) -> [i32; 3] {
    let value = properties.get("coordinate_3d").and_then(|v| v.as_array());
    let at = |i: usize| {
        value
            .and_then(|a| a.get(i))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32
    };
    [at(0), at(1), at(2)]
}

/// Undo `created` (mappings, areas, morphologies, region). Items already gone count as removed.
/// Returns what could not be removed.
async fn undo(state: &ApiState, created: &Created) -> Vec<String> {
    let connectome = state.connectome_service.as_ref();
    let mut failures = Vec::new();
    let mut note = |what: String, error: feagi_services::ServiceError| {
        if !is_not_found(&error) {
            tracing::error!(target: "feagi-api", "Block wiring undo could not remove {}: {}", what, error);
            failures.push(format!("{what} ({error})"));
        }
    };
    for (src, dst) in created.mappings.iter().rev() {
        if let Err(e) = connectome
            .update_cortical_mapping(src.clone(), dst.clone(), Vec::new())
            .await
        {
            note(format!("mapping {src} -> {dst}"), e);
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
    for id in created.regions.iter().rev() {
        if let Err(e) = connectome.delete_brain_region(id).await {
            note(format!("brain region {id}"), e);
        }
    }
    failures
}

/// Create the adapter. With `skip_existing`, a region, area or morphology that is already there
/// is left alone (used to finish restoring a half-removed adapter). Everything it creates is
/// recorded in `created` so the caller can undo it. Returns one report per mapping.
async fn create_adapter(
    state: &ApiState,
    spec: &AdapterSpec,
    skip_existing: bool,
    created: &mut Created,
) -> Result<Vec<serde_json::Value>, ApiError> {
    let connectome = state.connectome_service.as_ref();
    let genome_service = state.genome_service.as_ref();
    let exists_region = skip_existing
        && connectome
            .brain_region_exists(&spec.region_id)
            .await
            .unwrap_or(false);
    if !exists_region {
        connectome
            .create_brain_region(feagi_services::types::CreateBrainRegionParams {
                region_id: spec.region_id.clone(),
                name: spec.region_name.clone(),
                region_type: "Custom".to_string(),
                parent_id: Some(spec.parent_id.clone()),
                properties: Some(spec.properties.clone()),
            })
            .await
            .map_err(|e| {
                ApiError::internal(format!("Failed to create adapter brain region: {}", e))
            })?;
        created.regions.push(spec.region_id.clone());
    }

    let origin = region_origin(&spec.properties);
    let mut x_offset = origin[0];
    let mut params = Vec::new();
    for area in &spec.record.areas {
        let [w, h, d] = area.dimensions;
        let position = (x_offset, origin[1], origin[2]);
        x_offset = x_offset.saturating_add(w as i32).saturating_add(2);
        if skip_existing
            && connectome
                .cortical_area_exists(&area.cortical_id)
                .await
                .unwrap_or(false)
        {
            continue;
        }
        params.push(feagi_services::types::CreateCorticalAreaParams {
            cortical_id: area.cortical_id.clone(),
            name: area.name.clone(),
            dimensions: (w as usize, h as usize, d as usize),
            position,
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
            properties: Some(HashMap::from([(
                "parent_region_id".to_string(),
                serde_json::json!(spec.region_id),
            )])),
        });
    }
    if !params.is_empty() {
        created
            .areas
            .extend(params.iter().map(|p| p.cortical_id.clone()));
        genome_service
            .create_cortical_areas(params)
            .await
            .map_err(|e| ApiError::internal(format!("Failed to create adapter areas: {}", e)))?;
    }

    for morphology in &spec.record.morphologies {
        if skip_existing
            && connectome
                .get_morphologies()
                .await
                .map(|m| m.contains_key(&morphology.name))
                .unwrap_or(false)
        {
            continue;
        }
        connectome
            .create_morphology(morphology.name.clone(), morphology.to_morphology())
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to create morphology '{}': {}",
                    morphology.name, e
                ))
            })?;
        created.morphologies.push(morphology.name.clone());
    }

    let mut reports = Vec::new();
    for mapping in &spec.record.mappings {
        let synapse_count = connectome
            .update_cortical_mapping(
                mapping.src.clone(),
                mapping.dst.clone(),
                mapping.rules.clone(),
            )
            .await
            .map_err(|e| {
                ApiError::internal(format!(
                    "Failed to create mapping {} -> {}: {}",
                    mapping.src, mapping.dst, e
                ))
            })?;
        created
            .mappings
            .push((mapping.src.clone(), mapping.dst.clone()));
        reports.push(serde_json::json!({
            "src": mapping.src,
            "dst": mapping.dst,
            "synapse_count": synapse_count,
        }));
    }
    Ok(reports)
}

/// An existing adapter read back from its region.
struct ExistingAdapter {
    region_id: String,
    name: String,
    parent_id: String,
    properties: Properties,
    record: AdapterRecord,
}

async fn read_adapter(state: &ApiState, region_id: &str) -> Result<ExistingAdapter, ApiError> {
    let region = state
        .connectome_service
        .get_brain_region(region_id)
        .await
        .map_err(|e| {
            if is_not_found(&e) {
                ApiError::invalid_input(format!("Adapter region {} does not exist", region_id))
            } else {
                ApiError::internal(format!("Failed to read region {}: {}", region_id, e))
            }
        })?;
    let record = region
        .properties
        .get(ADAPTER_WIRING_KEY)
        .cloned()
        .and_then(|v| serde_json::from_value::<AdapterRecord>(v).ok())
        .filter(|_| region.properties.contains_key(ADAPTER_FOR_KEY))
        .ok_or_else(|| {
            ApiError::invalid_input(format!("Region {} is not an adapter region", region_id))
        })?;
    Ok(ExistingAdapter {
        region_id: region.region_id,
        name: region.name,
        parent_id: region.parent_id.unwrap_or_default(),
        properties: region.properties,
        record,
    })
}

fn spec_of(existing: &ExistingAdapter) -> AdapterSpec {
    AdapterSpec {
        region_id: existing.region_id.clone(),
        region_name: existing.name.clone(),
        parent_id: existing.parent_id.clone(),
        properties: existing.properties.clone(),
        record: existing.record.clone(),
    }
}

/// Remove an adapter's mappings, areas, morphologies and region. On a failure other than "already
/// gone", the adapter is put back (items still present are kept) and the error is returned.
async fn remove_adapter(state: &ApiState, existing: &ExistingAdapter) -> Result<(), ApiError> {
    let connectome = state.connectome_service.as_ref();
    let mut failure: Option<String> = None;
    let mut step = |what: String, result: Result<(), feagi_services::ServiceError>| {
        if let Err(e) = result {
            if !is_not_found(&e) && failure.is_none() {
                failure = Some(format!("{what} ({e})"));
            }
        }
    };
    for mapping in existing.record.mappings.iter().rev() {
        let r = connectome
            .update_cortical_mapping(mapping.src.clone(), mapping.dst.clone(), Vec::new())
            .await
            .map(|_| ());
        step(format!("mapping {} -> {}", mapping.src, mapping.dst), r);
    }
    for area in existing.record.areas.iter().rev() {
        let r = connectome.delete_cortical_area(&area.cortical_id).await;
        step(format!("cortical area {}", area.cortical_id), r);
    }
    for morphology in existing.record.morphologies.iter().rev() {
        let r = connectome.delete_morphology(&morphology.name).await;
        step(format!("morphology '{}'", morphology.name), r);
    }
    let r = connectome.delete_brain_region(&existing.region_id).await;
    step(format!("brain region {}", existing.region_id), r);
    let Some(failure) = failure else {
        return Ok(());
    };
    let mut created = Created::default();
    let restored = create_adapter(state, &spec_of(existing), true, &mut created).await;
    let mut message = format!("Could not remove the adapter: {failure}.");
    if let Err(e) = restored {
        message.push_str(&format!(" Restoring it also failed: {}.", e.message));
    }
    Err(ApiError::internal(message))
}

fn transition_lock(state: &ApiState) -> Result<tokio::sync::RwLockWriteGuard<'_, ()>, ApiError> {
    state.genome_transition_lock.try_write().map_err(|_| {
        ApiError::conflict("Another genome transition is in progress; wait for it to finish")
    })
}

/// Create (or replace) the adapter region for a block already in the brain.
///
/// Body: `{ "block_region_id", "adapter": {"region_name"?, "areas": [...]}, "morphologies": [...],
/// "mappings": [...], "stamp": {...}, "replaces_adapter_region_id"? }`. Validated like a fit plan
/// (pattern rows inside both ends, adapter areas are plain custom areas, morphology names unique
/// and free); any failure changes nothing. Creates one region, the adapter.
#[utoipa::path(post, path = "/v1/genome/block_wiring", tag = "genome")]
pub async fn post_block_wiring(
    State(state): State<ApiState>,
    Json(req): Json<BlockWiringRequest>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let _lock = transition_lock(&state)?;
    let connectome = state.connectome_service.as_ref();
    let block = connectome
        .get_brain_region(&req.block_region_id)
        .await
        .map_err(|e| {
            if is_not_found(&e) {
                ApiError::invalid_input(format!(
                    "Block region {} does not exist",
                    req.block_region_id
                ))
            } else {
                ApiError::internal(format!("Failed to read block region: {}", e))
            }
        })?;
    if !req.stamp.is_null() && !req.stamp.is_object() {
        return Err(ApiError::invalid_input("stamp must be an object"));
    }

    // The adapter being replaced must serve this block; what it holds is free for the new one.
    let old = match &req.replaces_adapter_region_id {
        Some(id) => {
            let old = read_adapter(&state, id).await?;
            if old.properties.get(ADAPTER_FOR_KEY).and_then(|v| v.as_str())
                != Some(req.block_region_id.as_str())
            {
                return Err(ApiError::invalid_input(format!(
                    "Adapter region {} is not an adapter for block region {}",
                    id, req.block_region_id
                )));
            }
            Some(old)
        }
        None => None,
    };

    let plan = FitPlan {
        areas: Vec::new(),
        adapter: Some(req.adapter.clone()),
        morphologies: req.morphologies.clone(),
        mappings: req.mappings.clone(),
    };
    let no_guest = HashMap::new();
    let mut host = read_plan_host(&state, &plan, &no_guest).await?;
    if let Some(old) = &old {
        for area in &old.record.areas {
            host.areas.remove(&area.cortical_id);
        }
        for morphology in &old.record.morphologies {
            host.morphologies.remove(&morphology.name);
        }
    }
    check_plan_wiring(&plan, &no_guest, &HashSet::new(), &host)?;

    // A mapping out of an existing host area replaces any rule already there, which an undo could
    // not restore. Pairs the replaced adapter owns are removed first, so they do not count.
    let old_pairs: HashSet<(&str, &str)> = old
        .iter()
        .flat_map(|o| o.record.mappings.iter())
        .map(|m| (m.src.as_str(), m.dst.as_str()))
        .collect();
    let adapter_ids: HashSet<&str> = req
        .adapter
        .areas
        .iter()
        .map(|a| a.cortical_id.as_str())
        .collect();
    for mapping in &req.mappings {
        if adapter_ids.contains(mapping.src.as_str())
            || old_pairs.contains(&(mapping.src.as_str(), mapping.dst.as_str()))
        {
            continue;
        }
        let properties = connectome
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

    let record = AdapterRecord {
        areas: req.adapter.areas.clone(),
        morphologies: req.morphologies.clone(),
        mappings: req.mappings.clone(),
    };
    let block_origin = region_origin(&block.properties);
    let block_width = block
        .properties
        .get("circuit_size")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|v| v.as_i64())
        .unwrap_or(6) as i32;
    let mut properties: Properties = HashMap::new();
    properties.insert(
        "coordinate_3d".to_string(),
        serde_json::json!([
            block_origin[0]
                .saturating_add(block_width)
                .saturating_add(4),
            block_origin[1],
            block_origin[2]
        ]),
    );
    properties.insert(
        ADAPTER_FOR_KEY.to_string(),
        serde_json::json!(req.block_region_id),
    );
    properties.insert(STAMP_KEY.to_string(), req.stamp.clone());
    properties.insert(
        ADAPTER_WIRING_KEY.to_string(),
        serde_json::to_value(&record)
            .map_err(|e| ApiError::internal(format!("Failed to encode the adapter record: {e}")))?,
    );
    let spec = AdapterSpec {
        region_id: Uuid::new_v4().to_string(),
        region_name: req
            .adapter
            .region_name
            .clone()
            .unwrap_or_else(|| format!("{} adapter", block.name)),
        parent_id: block
            .parent_id
            .clone()
            .ok_or_else(|| ApiError::invalid_input("The block region has no parent region"))?,
        properties,
        record,
    };

    if let Some(old) = &old {
        remove_adapter(&state, old).await?;
    }
    let mut created = Created::default();
    let reports = match create_adapter(&state, &spec, false, &mut created).await {
        Ok(reports) => reports,
        Err(mut error) => {
            let failures = undo(&state, &created).await;
            if !failures.is_empty() {
                error.message.push_str(&format!(
                    " Rollback could not remove: {}.",
                    failures.join("; ")
                ));
            }
            if let Some(old) = &old {
                let mut restored = Created::default();
                match create_adapter(&state, &spec_of(old), false, &mut restored).await {
                    Ok(_) => error.message.push_str(" The previous adapter was kept."),
                    Err(e) => error.message.push_str(&format!(
                        " Restoring the previous adapter failed: {}.",
                        e.message
                    )),
                }
            }
            return Err(error);
        }
    };

    bump_hashes();
    let mut response: HashMap<String, serde_json::Value> = HashMap::from([
        (
            "message".to_string(),
            serde_json::json!("Block wiring created"),
        ),
        (
            "adapter_region_id".to_string(),
            serde_json::json!(spec.region_id),
        ),
        (
            "adapter_area_count".to_string(),
            serde_json::json!(spec.record.areas.len()),
        ),
        (
            "morphology_count".to_string(),
            serde_json::json!(spec.record.morphologies.len()),
        ),
        (
            "mapping_count".to_string(),
            serde_json::json!(reports.len()),
        ),
        ("mappings".to_string(), serde_json::Value::Array(reports)),
    ]);
    if let Some(old) = old {
        response.insert(
            "replaced_adapter_region_id".to_string(),
            serde_json::json!(old.region_id),
        );
    }
    Ok(Json(response))
}

/// Remove an adapter region with its areas, morphologies and mappings.
///
/// Body: `{ "adapter_region_id" }`. The region must carry the adapter record this API wrote.
#[utoipa::path(post, path = "/v1/genome/block_wiring_removal", tag = "genome")]
pub async fn post_block_wiring_removal(
    State(state): State<ApiState>,
    Json(req): Json<BlockWiringRemovalRequest>,
) -> ApiResult<Json<HashMap<String, serde_json::Value>>> {
    let _lock = transition_lock(&state)?;
    let existing = read_adapter(&state, &req.adapter_region_id).await?;
    remove_adapter(&state, &existing).await?;
    bump_hashes();
    Ok(Json(HashMap::from([
        (
            "message".to_string(),
            serde_json::json!("Block wiring removed"),
        ),
        (
            "removed_area_count".to_string(),
            serde_json::json!(existing.record.areas.len()),
        ),
        (
            "removed_mapping_count".to_string(),
            serde_json::json!(existing.record.mappings.len()),
        ),
    ])))
}

/// Tell Brain Visualizer its cached regions, areas, morphologies and mappings changed.
fn bump_hashes() {
    let state_manager = feagi_state_manager::StateManager::instance();
    let state_manager = state_manager.read();
    state_manager.set_brain_regions_hash(state_manager.get_brain_regions_hash().wrapping_add(1));
    state_manager.set_cortical_areas_hash(state_manager.get_cortical_areas_hash().wrapping_add(1));
    state_manager.set_brain_geometry_hash(state_manager.get_brain_geometry_hash().wrapping_add(1));
    state_manager.set_morphologies_hash(state_manager.get_morphologies_hash().wrapping_add(1));
    state_manager
        .set_cortical_mappings_hash(state_manager.get_cortical_mappings_hash().wrapping_add(1));
}
