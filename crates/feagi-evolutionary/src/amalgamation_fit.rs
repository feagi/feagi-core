// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Fit a guest genome's cortical areas to the host before any area is created, and carry the wiring
that joins the fitted block to the rest of the brain.

A [`FitPlan`] lists guest cortical ids and the dimensions each must have. [`fit_areas`] is pure: it
returns a copy of the guest areas with those dimensions and changes nothing else. Creating the
areas at their fitted size avoids a post-creation resize, which rebuilds an area's neurons and
synapses.

A plan may also carry an adapter (new plain areas in a new brain region), morphologies (the
existing `patterns` type, exact coordinates only) and mappings between guest, adapter and host
areas. [`validate_plan`] checks all of it without touching a brain; the caller applies it only
when it passes, and again re-checks it against the live host at confirmation.

The plan is body-free data (area ids, names, dimensions, coordinates and mapping rules). Whoever
builds it knows the host and the robot; this module does not.

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use crate::runtime::{Morphology, MorphologyParameters, MorphologyType, PatternElement};
use feagi_structures::genomic::cortical_area::descriptors::CorticalAreaDimensions;
use feagi_structures::genomic::cortical_area::{CorticalArea, CorticalAreaType, CorticalID};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// One fitted area: the guest cortical id (base64, as in the guest genome) and its dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaFit {
    pub cortical_id: String,
    /// `[x, y, z]`; every value must be at least 1.
    pub dimensions: [u32; 3],
}

/// One new plain area of the adapter region.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterArea {
    /// Base64 cortical id of a Custom area (never IO, core, memory or modulator).
    pub cortical_id: String,
    pub name: String,
    /// `[x, y, z]`; every value must be at least 1.
    pub dimensions: [u32; 3],
}

/// New areas created in a new brain region, between the block and the host's IO areas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterPlan {
    /// Name of the new region; the caller picks a default when absent.
    #[serde(default)]
    pub region_name: Option<String>,
    pub areas: Vec<AdapterArea>,
}

/// A `patterns` morphology with exact coordinates: each row is `[[sx,sy,sz],[dx,dy,dz]]`, the
/// same JSON as the existing patterns type minus wildcards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanMorphology {
    pub name: String,
    pub patterns: Vec<[[i32; 3]; 2]>,
}

impl PlanMorphology {
    /// The runtime morphology this definition creates.
    pub fn to_morphology(&self) -> Morphology {
        Morphology {
            morphology_type: MorphologyType::Patterns,
            parameters: MorphologyParameters::Patterns {
                patterns: self
                    .patterns
                    .iter()
                    .map(|[src, dst]| {
                        [
                            src.iter().map(|v| PatternElement::Value(*v)).collect(),
                            dst.iter().map(|v| PatternElement::Value(*v)).collect(),
                        ]
                    })
                    .collect(),
            },
            class: "custom".to_string(),
        }
    }
}

/// One mapping to create. `src` and `dst` are a guest area id (as published, before the import
/// remaps it), an adapter area id, or an existing host area id. `rules` are mapping rules in the
/// shape `update_cortical_mapping` takes; each names a `morphology_id`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanMapping {
    pub src: String,
    pub dst: String,
    pub rules: Vec<serde_json::Value>,
}

/// What the guest's areas must look like on the host, and the wiring to add around them. Every
/// field but `areas` is optional; an absent field changes nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FitPlan {
    #[serde(default)]
    pub areas: Vec<AreaFit>,
    #[serde(default)]
    pub adapter: Option<AdapterPlan>,
    #[serde(default)]
    pub morphologies: Vec<PlanMorphology>,
    #[serde(default)]
    pub mappings: Vec<PlanMapping>,
}

impl FitPlan {
    /// True when the plan adds an adapter, morphologies or mappings (not only dimensions).
    pub fn has_wiring(&self) -> bool {
        self.adapter.is_some() || !self.morphologies.is_empty() || !self.mappings.is_empty()
    }
}

/// Why a plan was refused. Nothing is changed when one is returned.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FitPlanError {
    #[error("fit plan names guest area '{0}', which is not in the guest genome")]
    UnknownArea(String),
    #[error("fit plan gives area '{cortical_id}' a zero dimension {dimensions:?}")]
    ZeroDimension {
        cortical_id: String,
        dimensions: [u32; 3],
    },
    #[error("fit plan targets IO area '{0}'; IO areas keep their size")]
    IoArea(String),
    #[error("fit plan names guest area '{0}' more than once")]
    DuplicateArea(String),
    #[error("fit plan has an invalid cortical id '{0}'")]
    InvalidCorticalId(String),
    #[error("adapter lists no areas")]
    EmptyAdapter,
    #[error("adapter area '{0}' is an IO area; adapter areas are plain custom areas")]
    AdapterIoArea(String),
    #[error("adapter area '{0}' is not a custom area; adapter areas are plain custom areas")]
    AdapterNotCustom(String),
    #[error("adapter area '{0}' has a zero dimension or an empty name")]
    AdapterBadArea(String),
    #[error(
        "adapter area id '{0}' is listed twice, or is also a guest area or an existing host area"
    )]
    AdapterIdClash(String),
    #[error("morphology '{0}' is defined twice in the plan")]
    DuplicateMorphology(String),
    #[error("morphology '{0}' has no name or no rows")]
    EmptyMorphology(String),
    #[error("morphology '{0}' clashes with a guest or host morphology of the same name")]
    MorphologyClash(String),
    #[error("mapping end '{0}' is not a guest area, an adapter area or an existing host area")]
    UnknownEnd(String),
    #[error("mapping end '{0}' is a guest area that is not created fresh (only custom and memory guest areas can be wired by a plan; name the host area instead)")]
    GuestEndNotFresh(String),
    #[error("mapping {src} -> {dst} is listed twice")]
    DuplicateMapping { src: String, dst: String },
    #[error("mapping {src} -> {dst} is already wired by the guest genome")]
    GuestMappingConflict { src: String, dst: String },
    #[error("mapping {src} -> {dst} has no rules")]
    EmptyRules { src: String, dst: String },
    #[error("mapping {src} -> {dst} has a bad rule: {reason}")]
    BadRule {
        src: String,
        dst: String,
        reason: String,
    },
    #[error("{0}")]
    PatternOutOfBounds(Box<PatternBounds>),
}

/// A `patterns` row that falls outside the area at one of its ends.
#[derive(Debug, PartialEq, Eq)]
pub struct PatternBounds {
    pub morphology: String,
    pub row: usize,
    /// `"source"` or `"destination"`.
    pub end: &'static str,
    pub coordinate: [i32; 3],
    pub area: String,
    pub dimensions: [u32; 3],
    pub src: String,
    pub dst: String,
}

impl std::fmt::Display for PatternBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "morphology '{}' row {}: {} coordinate {:?} is outside the {} area {} of size {:?} \
             (mapping {} -> {})",
            self.morphology,
            self.row,
            self.end,
            self.coordinate,
            self.end,
            self.area,
            self.dimensions,
            self.src,
            self.dst
        )
    }
}

/// Return `areas` with the plan's dimensions applied. Validates the whole plan first; on any error
/// nothing is returned. An empty plan returns an unchanged copy.
pub fn fit_areas(
    areas: &HashMap<CorticalID, CorticalArea>,
    plan: &FitPlan,
) -> Result<HashMap<CorticalID, CorticalArea>, FitPlanError> {
    let by_b64: HashMap<String, CorticalID> =
        areas.keys().map(|id| (id.as_base_64(), *id)).collect();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut resolved: Vec<(CorticalID, CorticalAreaDimensions)> = Vec::new();
    for fit in &plan.areas {
        let id = by_b64
            .get(&fit.cortical_id)
            .ok_or_else(|| FitPlanError::UnknownArea(fit.cortical_id.clone()))?;
        if !seen.insert(fit.cortical_id.as_str()) {
            return Err(FitPlanError::DuplicateArea(fit.cortical_id.clone()));
        }
        if matches!(
            areas[id].cortical_type,
            CorticalAreaType::BrainInput(_) | CorticalAreaType::BrainOutput(_)
        ) {
            return Err(FitPlanError::IoArea(fit.cortical_id.clone()));
        }
        let [x, y, z] = fit.dimensions;
        let dims =
            CorticalAreaDimensions::new(x, y, z).map_err(|_| FitPlanError::ZeroDimension {
                cortical_id: fit.cortical_id.clone(),
                dimensions: fit.dimensions,
            })?;
        resolved.push((*id, dims));
    }
    let mut fitted = areas.clone();
    for (id, dims) in resolved {
        if let Some(area) = fitted.get_mut(&id) {
            area.dimensions = dims;
        }
    }
    Ok(fitted)
}

/// Everything outside the plan that its validation reads.
pub struct PlanContext<'a> {
    /// Guest areas after [`fit_areas`] (fitted dimensions, published ids).
    pub guest_areas: &'a HashMap<CorticalID, CorticalArea>,
    /// Names of the guest genome's morphologies.
    pub guest_morphologies: &'a HashSet<String>,
    /// Dimensions of every id from [`plan_host_lookup_ids`] that exists on the host.
    pub host_areas: &'a HashMap<String, [u32; 3]>,
    /// Names of the host's morphologies.
    pub host_morphologies: &'a HashSet<String>,
}

/// Ids the caller must look up on the host before validating: adapter ids (they must not exist)
/// and mapping ends that are neither guest nor adapter areas (they must exist). Sorted, unique.
pub fn plan_host_lookup_ids(
    guest_areas: &HashMap<CorticalID, CorticalArea>,
    plan: &FitPlan,
) -> Vec<String> {
    let guest: HashSet<String> = guest_areas.keys().map(CorticalID::as_base_64).collect();
    let adapter: HashSet<&str> = plan
        .adapter
        .iter()
        .flat_map(|a| a.areas.iter().map(|x| x.cortical_id.as_str()))
        .collect();
    let mut ids: Vec<String> = adapter.iter().map(|id| id.to_string()).collect();
    for mapping in &plan.mappings {
        for end in [&mapping.src, &mapping.dst] {
            if !guest.contains(end) && !adapter.contains(end.as_str()) {
                ids.push(end.clone());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Validate the adapter, morphologies and mappings of `plan` (not its `areas`; [`fit_areas`] does
/// those). Pure: reads only `ctx`. Run it when the plan is queued and again at confirmation with
/// freshly read host data.
pub fn validate_plan(ctx: &PlanContext, plan: &FitPlan) -> Result<(), FitPlanError> {
    let guest_ids: HashSet<String> = ctx.guest_areas.keys().map(CorticalID::as_base_64).collect();

    // Adapter areas: plain custom areas with ids nobody else uses.
    let mut adapter_dims: HashMap<&str, [u32; 3]> = HashMap::new();
    if let Some(adapter) = &plan.adapter {
        if adapter.areas.is_empty() {
            return Err(FitPlanError::EmptyAdapter);
        }
        for area in &adapter.areas {
            let id = CorticalID::try_from_base_64(&area.cortical_id)
                .map_err(|_| FitPlanError::InvalidCorticalId(area.cortical_id.clone()))?;
            match id.as_cortical_type() {
                Ok(CorticalAreaType::Custom(_)) => {}
                Ok(CorticalAreaType::BrainInput(_) | CorticalAreaType::BrainOutput(_)) => {
                    return Err(FitPlanError::AdapterIoArea(area.cortical_id.clone()))
                }
                _ => return Err(FitPlanError::AdapterNotCustom(area.cortical_id.clone())),
            }
            if area.name.trim().is_empty() || area.dimensions.contains(&0) {
                return Err(FitPlanError::AdapterBadArea(area.cortical_id.clone()));
            }
            if adapter_dims
                .insert(area.cortical_id.as_str(), area.dimensions)
                .is_some()
                || guest_ids.contains(&area.cortical_id)
                || ctx.host_areas.contains_key(&area.cortical_id)
            {
                return Err(FitPlanError::AdapterIdClash(area.cortical_id.clone()));
            }
        }
    }

    // Morphologies: unique names, no clash with guest or host, at least one row.
    let mut morphologies: HashMap<&str, &PlanMorphology> = HashMap::new();
    for morphology in &plan.morphologies {
        if morphology.name.trim().is_empty() || morphology.patterns.is_empty() {
            return Err(FitPlanError::EmptyMorphology(morphology.name.clone()));
        }
        if morphologies
            .insert(morphology.name.as_str(), morphology)
            .is_some()
        {
            return Err(FitPlanError::DuplicateMorphology(morphology.name.clone()));
        }
        if ctx.guest_morphologies.contains(&morphology.name)
            || ctx.host_morphologies.contains(&morphology.name)
        {
            return Err(FitPlanError::MorphologyClash(morphology.name.clone()));
        }
    }

    // Mappings: known ends, rules that name a morphology, rows inside both ends' dimensions.
    let end_dims = |end: &str| -> Result<[u32; 3], FitPlanError> {
        if let Some(dims) = adapter_dims.get(end) {
            return Ok(*dims);
        }
        if guest_ids.contains(end) {
            let id = CorticalID::try_from_base_64(end)
                .map_err(|_| FitPlanError::InvalidCorticalId(end.to_string()))?;
            let area = &ctx.guest_areas[&id];
            if !matches!(
                area.cortical_type,
                CorticalAreaType::Custom(_) | CorticalAreaType::Memory(_)
            ) {
                return Err(FitPlanError::GuestEndNotFresh(end.to_string()));
            }
            let d = &area.dimensions;
            return Ok([d.width, d.height, d.depth]);
        }
        ctx.host_areas
            .get(end)
            .copied()
            .ok_or_else(|| FitPlanError::UnknownEnd(end.to_string()))
    };
    let mut pairs: HashSet<(&str, &str)> = HashSet::new();
    for mapping in &plan.mappings {
        let (src, dst) = (mapping.src.as_str(), mapping.dst.as_str());
        let src_dims = end_dims(src)?;
        let dst_dims = end_dims(dst)?;
        let pair_err = || FitPlanError::DuplicateMapping {
            src: mapping.src.clone(),
            dst: mapping.dst.clone(),
        };
        if !pairs.insert((src, dst)) {
            return Err(pair_err());
        }
        if let Ok(id) = CorticalID::try_from_base_64(src) {
            let wired = ctx
                .guest_areas
                .get(&id)
                .and_then(|a| a.properties.get("cortical_mapping_dst"))
                .and_then(|v| v.as_object())
                .is_some_and(|dsts| dsts.contains_key(dst));
            if wired {
                return Err(FitPlanError::GuestMappingConflict {
                    src: mapping.src.clone(),
                    dst: mapping.dst.clone(),
                });
            }
        }
        if mapping.rules.is_empty() {
            return Err(FitPlanError::EmptyRules {
                src: mapping.src.clone(),
                dst: mapping.dst.clone(),
            });
        }
        for rule in &mapping.rules {
            let bad = |reason: &str| FitPlanError::BadRule {
                src: mapping.src.clone(),
                dst: mapping.dst.clone(),
                reason: reason.to_string(),
            };
            let name = rule
                .as_object()
                .ok_or_else(|| bad("a rule must be an object"))?
                .get("morphology_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| bad("a rule must name a string 'morphology_id'"))?;
            // A morphology not defined by the plan is the host's or the guest's; the caller
            // checks it exists when it applies the plan.
            let Some(morphology) = morphologies.get(name) else {
                continue;
            };
            for (row, [from, to]) in morphology.patterns.iter().enumerate() {
                for (end, coordinate, area, dims) in [
                    ("source", from, src, src_dims),
                    ("destination", to, dst, dst_dims),
                ] {
                    let inside = coordinate
                        .iter()
                        .zip(dims.iter())
                        .all(|(c, d)| *c >= 0 && (*c as i64) < i64::from(*d));
                    if !inside {
                        return Err(FitPlanError::PatternOutOfBounds(Box::new(PatternBounds {
                            morphology: name.to_string(),
                            row,
                            end,
                            coordinate: *coordinate,
                            area: area.to_string(),
                            dimensions: dims,
                            src: mapping.src.clone(),
                            dst: mapping.dst.clone(),
                        })));
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use feagi_structures::genomic::cortical_area::CustomCorticalType;
    use feagi_structures::genomic::descriptors::GenomeCoordinate3D;

    fn custom_id(tag: u8) -> CorticalID {
        let mut bytes = [b'a'; CorticalID::CORTICAL_ID_LENGTH];
        bytes[0] = b'c';
        bytes[1] = tag;
        CorticalID::try_from_bytes(&bytes).expect("custom id")
    }

    fn area(id: CorticalID, w: u32, h: u32, d: u32) -> CorticalArea {
        CorticalArea::new(
            id,
            0,
            "area".to_string(),
            CorticalAreaDimensions::new(w, h, d).expect("dims"),
            GenomeCoordinate3D::new(0, 0, 0),
            CorticalAreaType::Custom(CustomCorticalType::LeakyIntegrateFire),
        )
        .expect("area")
    }

    fn areas() -> (HashMap<CorticalID, CorticalArea>, CorticalID, CorticalID) {
        let (a, b) = (custom_id(b'a'), custom_id(b'b'));
        (
            HashMap::from([(a, area(a, 12, 1, 1)), (b, area(b, 3, 1, 1))]),
            a,
            b,
        )
    }

    fn fit(id: &CorticalID, dimensions: [u32; 3]) -> AreaFit {
        AreaFit {
            cortical_id: id.as_base_64(),
            dimensions,
        }
    }

    #[test]
    fn fit_applies_dimensions_and_leaves_others() {
        let (areas, a, b) = areas();
        let plan = FitPlan {
            areas: vec![fit(&a, [14, 1, 1])],
            ..Default::default()
        };
        let fitted = fit_areas(&areas, &plan).expect("fit");
        assert_eq!(fitted[&a].dimensions.width, 14);
        assert_eq!(fitted[&b].dimensions, areas[&b].dimensions);
        assert_eq!(areas[&a].dimensions.width, 12, "input is not modified");
    }

    #[test]
    fn fit_empty_plan_is_identity() {
        let (areas, a, _) = areas();
        let fitted = fit_areas(&areas, &FitPlan::default()).expect("fit");
        assert_eq!(fitted[&a].dimensions, areas[&a].dimensions);
        assert_eq!(fitted.len(), areas.len());
    }

    #[test]
    fn fit_rejects_unknown_area() {
        let (areas, _, _) = areas();
        let ghost = custom_id(b'z');
        let plan = FitPlan {
            areas: vec![fit(&ghost, [2, 1, 1])],
            ..Default::default()
        };
        assert_eq!(
            fit_areas(&areas, &plan).unwrap_err(),
            FitPlanError::UnknownArea(ghost.as_base_64())
        );
    }

    #[test]
    fn fit_rejects_zero_dimension_and_changes_nothing() {
        let (areas, a, b) = areas();
        let plan = FitPlan {
            areas: vec![fit(&a, [14, 1, 1]), fit(&b, [0, 1, 1])],
            ..Default::default()
        };
        assert!(matches!(
            fit_areas(&areas, &plan).unwrap_err(),
            FitPlanError::ZeroDimension { .. }
        ));
        assert_eq!(areas[&a].dimensions.width, 12);
    }

    #[test]
    fn fit_rejects_duplicate_area() {
        let (areas, a, _) = areas();
        let plan = FitPlan {
            areas: vec![fit(&a, [14, 1, 1]), fit(&a, [15, 1, 1])],
            ..Default::default()
        };
        assert_eq!(
            fit_areas(&areas, &plan).unwrap_err(),
            FitPlanError::DuplicateArea(a.as_base_64())
        );
    }

    #[test]
    fn fit_rejects_io_area() {
        use feagi_structures::genomic::cortical_area::IOCorticalAreaConfigurationFlag;
        let (mut areas, _, _) = areas();
        let io_flag = IOCorticalAreaConfigurationFlag::Boolean;
        let io_id = io_flag.as_io_cortical_id(true, *b"tst", 0u8.into(), 0u8.into());
        let mut io = area(io_id, 4, 1, 1);
        io.cortical_type = CorticalAreaType::BrainOutput(io_flag);
        areas.insert(io_id, io);
        let plan = FitPlan {
            areas: vec![fit(&io_id, [6, 1, 1])],
            ..Default::default()
        };
        assert_eq!(
            fit_areas(&areas, &plan).unwrap_err(),
            FitPlanError::IoArea(io_id.as_base_64())
        );
    }

    // ---- wiring: adapter, morphologies, mappings --------------------------------------------

    fn adapter_id(tag: u8) -> String {
        custom_id(tag).as_base_64()
    }

    fn host_id() -> String {
        // A host area id the guest does not have.
        custom_id(b'h').as_base_64()
    }

    fn rows(n: i32) -> Vec<[[i32; 3]; 2]> {
        (0..n).map(|i| [[i, 0, 0], [i, 0, 0]]).collect()
    }

    fn wiring(src_dims: u32, adapter_w: u32, host_w: u32) -> (FitPlan, HashMap<String, [u32; 3]>) {
        let (a, _) = (custom_id(b'a'), ());
        let plan = FitPlan {
            areas: vec![fit(&a, [src_dims, 1, 1])],
            adapter: Some(AdapterPlan {
                region_name: None,
                areas: vec![AdapterArea {
                    cortical_id: adapter_id(b'x'),
                    name: "adapt".to_string(),
                    dimensions: [adapter_w, 1, 1],
                }],
            }),
            morphologies: vec![PlanMorphology {
                name: "one_to_one".to_string(),
                patterns: rows(src_dims as i32),
            }],
            mappings: vec![
                PlanMapping {
                    src: a.as_base_64(),
                    dst: adapter_id(b'x'),
                    rules: vec![serde_json::json!({ "morphology_id": "one_to_one" })],
                },
                PlanMapping {
                    src: adapter_id(b'x'),
                    dst: host_id(),
                    rules: vec![serde_json::json!({ "morphology_id": "one_to_one" })],
                },
            ],
        };
        (plan, HashMap::from([(host_id(), [host_w, 1, 1])]))
    }

    fn check(plan: &FitPlan, host: &HashMap<String, [u32; 3]>) -> Result<(), FitPlanError> {
        let (areas, a, _) = areas();
        let fitted = fit_areas(&areas, plan).expect("fit");
        let _ = a;
        validate_plan(
            &PlanContext {
                guest_areas: &fitted,
                guest_morphologies: &HashSet::from(["projector".to_string()]),
                host_areas: host,
                host_morphologies: &HashSet::from(["host_morph".to_string()]),
            },
            plan,
        )
    }

    #[test]
    fn plan_with_wiring_validates_and_lists_host_lookups() {
        let (plan, host) = wiring(14, 14, 14);
        check(&plan, &host).expect("valid");
        let (areas, _, _) = areas();
        let mut expected = vec![adapter_id(b'x'), host_id()];
        expected.sort();
        assert_eq!(plan_host_lookup_ids(&areas, &plan), expected);
        assert!(plan.has_wiring() && !FitPlan::default().has_wiring());
        let json = serde_json::to_string(&plan).expect("json");
        assert_eq!(serde_json::from_str::<FitPlan>(&json).expect("round"), plan);
    }

    #[test]
    fn depth_mismatch_between_strip_and_host_area_is_refused() {
        // 14 rows against a 13-wide host area: row 13 falls outside the destination.
        let (plan, host) = wiring(14, 14, 13);
        match check(&plan, &host).unwrap_err() {
            FitPlanError::PatternOutOfBounds(bounds) => {
                assert_eq!((bounds.row, bounds.end), (13, "destination"))
            }
            other => panic!("{other}"),
        }
    }

    #[test]
    fn bad_wiring_plans_are_refused() {
        let (good, host) = wiring(14, 14, 14);
        let mut unknown = good.clone();
        unknown.mappings[1].dst = adapter_id(b'q');
        assert!(matches!(
            check(&unknown, &host).unwrap_err(),
            FitPlanError::UnknownEnd(_)
        ));
        let mut dup_morph = good.clone();
        dup_morph
            .morphologies
            .push(dup_morph.morphologies[0].clone());
        assert!(matches!(
            check(&dup_morph, &host).unwrap_err(),
            FitPlanError::DuplicateMorphology(_)
        ));
        for name in ["projector", "host_morph"] {
            let mut clash = good.clone();
            clash.morphologies[0].name = name.to_string();
            assert!(matches!(
                check(&clash, &host).unwrap_err(),
                FitPlanError::MorphologyClash(_)
            ));
        }
        let mut io = good.clone();
        io.adapter.as_mut().unwrap().areas[0].cortical_id = {
            use feagi_structures::genomic::cortical_area::IOCorticalAreaConfigurationFlag as F;
            F::Boolean
                .as_io_cortical_id(true, *b"tst", 0u8.into(), 0u8.into())
                .as_base_64()
        };
        assert!(matches!(
            check(&io, &host).unwrap_err(),
            FitPlanError::AdapterIoArea(_)
        ));
        let mut on_host = good.clone();
        on_host.adapter.as_mut().unwrap().areas[0].cortical_id = host_id();
        assert!(matches!(
            check(&on_host, &host).unwrap_err(),
            FitPlanError::AdapterIdClash(_)
        ));
        let mut dup_map = good.clone();
        dup_map.mappings.push(dup_map.mappings[0].clone());
        assert!(matches!(
            check(&dup_map, &host).unwrap_err(),
            FitPlanError::DuplicateMapping { .. }
        ));
        let mut no_rules = good.clone();
        no_rules.mappings[0].rules.clear();
        assert!(matches!(
            check(&no_rules, &host).unwrap_err(),
            FitPlanError::EmptyRules { .. }
        ));
        let mut bad_rule = good;
        bad_rule.mappings[0].rules = vec![serde_json::json!({ "no_morphology": 1 })];
        assert!(matches!(
            check(&bad_rule, &host).unwrap_err(),
            FitPlanError::BadRule { .. }
        ));
    }

    #[test]
    fn wildcard_rows_do_not_parse() {
        let bad = serde_json::json!({ "name": "w", "patterns": [[["*", 0, 0], [0, 0, 0]]] });
        assert!(serde_json::from_value::<PlanMorphology>(bad).is_err());
    }
}
