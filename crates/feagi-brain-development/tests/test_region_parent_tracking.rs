// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! # Brain Region Parent Tracking Tests
//!
//! Tests the parent-child relationship tracking with UUID-based RegionID:
//! - Root region created with UUID RegionID
//! - Subregions reference parent by UUID string
//! - Parent lookup works correctly with UUID keys
//! - Regression test for "Parent region does not exist" bug

use feagi_brain_development::models::brain_region_hierarchy::BrainRegionHierarchy;
use feagi_brain_development::{ConnectomeManager, CorticalArea, CorticalID, Neuroembryogenesis};
use feagi_evolutionary::create_genome_with_core_morphologies;
use feagi_npu_burst_engine::RustNPU;
use feagi_npu_burst_engine::TracingMutex;
use feagi_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_structures::genomic::cortical_area::CorticalAreaDimensions;
use parking_lot::RwLock;
use std::sync::Arc;

/// Helper to create a root region with UUID
fn create_root_region() -> BrainRegion {
    let root_region_id = RegionID::new();
    BrainRegion::new(
        root_region_id,
        "Root Brain Region".to_string(),
        RegionType::Undefined,
    )
    .expect("Failed to create root region")
}

/// Helper to create a subregion with UUID (no parent - parent is set when adding to hierarchy)
fn create_subregion() -> BrainRegion {
    let subregion_id = RegionID::new();
    BrainRegion::new(subregion_id, "Subregion".to_string(), RegionType::Undefined)
        .expect("Failed to create subregion")
}

// ═══════════════════════════════════════════════════════════
// Test 1: Root region with UUID can be stored and retrieved
// ═══════════════════════════════════════════════════════════

#[test]
fn test_root_region_uuid_storage() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root region with UUID
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();

    // Add to hierarchy (no parent for root)
    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    // Retrieve by UUID string
    let retrieved = hierarchy.get_region(&root_id_str);
    assert!(
        retrieved.is_some(),
        "Root region should be retrievable by UUID string"
    );
    assert_eq!(retrieved.unwrap().region_id, root_id);
}

// ═══════════════════════════════════════════════════════════
// Test: Omitted parent attaches under existing root (not a second top-level node)
// ═══════════════════════════════════════════════════════════

#[test]
fn test_omitted_parent_defaults_to_root_when_root_exists() {
    let mut hierarchy = BrainRegionHierarchy::new();

    let root_region = create_root_region();
    let root_id_str = root_region.region_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    let sub = create_subregion();
    let sub_id_str = sub.region_id.to_string();

    hierarchy
        .add_region(sub, None)
        .expect("Second region with omitted parent should attach under root");

    assert_eq!(
        hierarchy.get_parent(&sub_id_str),
        Some(&root_id_str),
        "Omitted parent must resolve to root_id, not a sibling of root"
    );
}

// ═══════════════════════════════════════════════════════════
// Test 2: Subregion can reference parent by UUID
// ═══════════════════════════════════════════════════════════

#[test]
fn test_subregion_parent_uuid_reference() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root region
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    // Create subregion
    let subregion = create_subregion();
    let subregion_id = subregion.region_id;
    let subregion_id_str = subregion_id.to_string();

    // Add subregion with parent reference
    hierarchy
        .add_region(subregion, Some(root_id_str.clone()))
        .expect("Failed to add subregion");

    // Verify parent relationship (stored in hierarchy, not in BrainRegion)
    let parent_id_opt = hierarchy.get_parent(&subregion_id_str);
    assert!(parent_id_opt.is_some(), "Subregion should have a parent");
    assert_eq!(
        parent_id_opt.unwrap(),
        &root_id_str,
        "Parent ID should match root region UUID"
    );
}

// ═══════════════════════════════════════════════════════════
// Test 3: Parent lookup works with UUID string keys
// ═══════════════════════════════════════════════════════════

#[test]
fn test_parent_lookup_with_uuid_keys() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root region
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    // Create subregion
    let subregion = create_subregion();
    let subregion_id = subregion.region_id;
    let subregion_id_str = subregion_id.to_string();

    hierarchy
        .add_region(subregion, Some(root_id_str.clone()))
        .expect("Failed to add subregion");

    // Get parent ID using UUID string
    let parent_id_opt = hierarchy.get_parent(&subregion_id_str);
    assert!(parent_id_opt.is_some(), "Parent should be retrievable");
    let parent_id_str = parent_id_opt.unwrap();
    assert_eq!(parent_id_str, &root_id_str, "Parent should be root region");

    // Get the actual parent region
    let parent_region = hierarchy.get_region(parent_id_str);
    assert!(parent_region.is_some(), "Parent region should exist");
    assert_eq!(parent_region.unwrap().region_id.to_string(), root_id_str);
}

// ═══════════════════════════════════════════════════════════
// Test 4: Multiple subregions with same parent
// ═══════════════════════════════════════════════════════════

#[test]
fn test_multiple_subregions_same_parent() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root region
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    // Create multiple subregions
    for i in 0..3 {
        let subregion = create_subregion();
        let subregion_id_str = subregion.region_id.to_string();

        hierarchy
            .add_region(subregion, Some(root_id_str.clone()))
            .unwrap_or_else(|_| panic!("Failed to add subregion {}", i));

        // Verify each can find its parent
        let parent_id_opt = hierarchy.get_parent(&subregion_id_str);
        assert!(
            parent_id_opt.is_some(),
            "Subregion {} should have parent",
            i
        );
        let parent_id_str = parent_id_opt.unwrap();
        assert_eq!(
            parent_id_str, &root_id_str,
            "Subregion {} parent should be root",
            i
        );

        // Get the actual parent region
        let parent_region = hierarchy.get_region(parent_id_str);
        assert!(
            parent_region.is_some(),
            "Subregion {} parent region should exist",
            i
        );
    }
}

// ═══════════════════════════════════════════════════════════
// Test 5: Deep hierarchy (grandchild)
// ═══════════════════════════════════════════════════════════

#[test]
fn test_deep_hierarchy_grandchild() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();
    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root");

    // Create child
    let child_region = create_subregion();
    let child_id = child_region.region_id;
    let child_id_str = child_id.to_string();
    hierarchy
        .add_region(child_region, Some(root_id_str.clone()))
        .expect("Failed to add child");

    // Create grandchild
    let grandchild_region = create_subregion();
    let grandchild_id_str = grandchild_region.region_id.to_string();
    hierarchy
        .add_region(grandchild_region, Some(child_id_str.clone()))
        .expect("Failed to add grandchild");

    // Verify grandchild can find parent (child)
    let parent_id_opt = hierarchy.get_parent(&grandchild_id_str);
    assert!(parent_id_opt.is_some(), "Grandchild should have parent");
    let parent_id_str = parent_id_opt.unwrap();
    assert_eq!(
        parent_id_str, &child_id_str,
        "Grandchild parent should be child"
    );

    // Verify child can find parent (root)
    let root_parent_opt = hierarchy.get_parent(&child_id_str);
    assert!(root_parent_opt.is_some(), "Child should have parent");
    let root_parent_id_str = root_parent_opt.unwrap();
    assert_eq!(
        root_parent_id_str, &root_id_str,
        "Child parent should be root"
    );
}

// ═══════════════════════════════════════════════════════════
// Test 6: Regression test - "Parent region does not exist" bug
// ═══════════════════════════════════════════════════════════

#[test]
fn test_regression_parent_region_exists() {
    // This test specifically addresses the bug where:
    // - Root region created with UUID
    // - Stored with UUID string as key
    // - Subregion references parent by UUID string
    // - Parent lookup should succeed

    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root with UUID (as neuroembryogenesis does)
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();

    // Store with UUID string as key (as ConnectomeManager does)
    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root");

    // Create subregion that references parent
    let subregion = create_subregion();
    let subregion_id_str = subregion.region_id.to_string();

    hierarchy
        .add_region(subregion, Some(root_id_str.clone()))
        .expect("Failed to add subregion");

    // This should NOT fail with "Parent region does not exist"
    let parent_id_opt = hierarchy.get_parent(&subregion_id_str);
    assert!(
        parent_id_opt.is_some(),
        "CRITICAL: Parent lookup should succeed - this was the bug!"
    );

    let parent_id_str = parent_id_opt.unwrap();
    assert_eq!(parent_id_str, &root_id_str, "Parent should be root region");

    // Verify we can get the actual parent region
    let parent_region = hierarchy.get_region(parent_id_str);
    assert!(parent_region.is_some(), "Parent region should exist");
    assert_eq!(parent_region.unwrap().region_id.to_string(), root_id_str);
}

// ═══════════════════════════════════════════════════════════
// Test 7: Verify regions are stored by UUID string, not hardcoded "root"
// ═══════════════════════════════════════════════════════════

#[test]
fn test_regions_stored_by_uuid_not_hardcoded() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root with UUID
    let root_region = create_root_region();
    let root_id = root_region.region_id;
    let root_id_str = root_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root");

    // Verify we CANNOT retrieve by hardcoded "root" string
    let by_hardcoded = hierarchy.get_region("root");
    assert!(
        by_hardcoded.is_none(),
        "Should NOT be retrievable by hardcoded 'root' string"
    );

    // But CAN retrieve by UUID string
    let by_uuid = hierarchy.get_region(&root_id_str);
    assert!(by_uuid.is_some(), "Should be retrievable by UUID string");
    assert_eq!(by_uuid.unwrap().region_id.to_string(), root_id_str);
}

/// Isolated manager so this test does not share ConnectomeManager::instance() with others.
fn isolated_neuroembryogenesis() -> (Neuroembryogenesis, Arc<RwLock<ConnectomeManager>>) {
    let runtime = feagi_npu_runtime::StdRuntime;
    let backend = feagi_npu_burst_engine::backend::CPUBackend::new();
    let npu_result =
        RustNPU::new(runtime, backend, 1_000_000, 10_000_000, 10).expect("Failed to create NPU");
    let npu = Arc::new(TracingMutex::new(
        feagi_npu_burst_engine::DynamicNPU::F32(npu_result),
        "NestedRegionTestNPU",
    ));
    let manager = Arc::new(RwLock::new(ConnectomeManager::new_for_testing_with_npu(
        npu,
    )));
    let neuro = Neuroembryogenesis::new(manager.clone());
    (neuro, manager)
}

fn region_with_parent(name: &str, parent_id: Option<&str>) -> BrainRegion {
    let mut region = BrainRegion::new(RegionID::new(), name.to_string(), RegionType::Undefined)
        .expect("Failed to create region");
    if let Some(parent) = parent_id {
        region.add_property("parent_region_id".to_string(), serde_json::json!(parent));
    }
    region
}

// ═══════════════════════════════════════════════════════════
// Integration: nested region load must not depend on HashMap order
// ═══════════════════════════════════════════════════════════

#[test]
fn test_develop_from_genome_accepts_nested_grandchild_region() {
    let (mut neuro, manager) = isolated_neuroembryogenesis();

    let mut genome = create_genome_with_core_morphologies(
        "nested_region_genome".to_string(),
        "Nested Region Genome".to_string(),
    );

    let cortical_id = CorticalID::try_from_bytes(b"cst_nest").unwrap();
    let cortical_type = cortical_id
        .as_cortical_type()
        .expect("Failed to get cortical type");
    let area = CorticalArea::new(
        cortical_id,
        0,
        "Nested Test Area".to_string(),
        CorticalAreaDimensions::new(2, 2, 1).unwrap(),
        (0, 0, 0).into(),
        cortical_type,
    )
    .expect("Failed to create cortical area");
    genome.cortical_areas.insert(cortical_id, area);

    let root = region_with_parent("Root Brain Region", None);
    let root_id = root.region_id.to_string();
    let parent = region_with_parent("Look for people", Some(&root_id));
    let parent_id = parent.region_id.to_string();
    let grandchild = region_with_parent("Wave", Some(&parent_id));
    let grandchild_id = grandchild.region_id.to_string();

    // Insert grandchild first so a HashMap-unlucky walk would try Wave before its parent.
    genome
        .brain_regions
        .insert(grandchild_id.clone(), grandchild);
    genome.brain_regions.insert(parent_id.clone(), parent);
    genome.brain_regions.insert(root_id.clone(), root);

    neuro
        .develop_from_genome(&genome)
        .expect("Nested region genome must develop without Parent-region-does-not-exist");

    let loaded = manager.read();
    assert_eq!(
        loaded.get_brain_region_ids().len(),
        3,
        "root, parent, and grandchild must all be registered"
    );
    assert_eq!(
        loaded
            .get_brain_region_hierarchy()
            .get_parent(&parent_id)
            .map(String::as_str),
        Some(root_id.as_str())
    );
    assert_eq!(
        loaded
            .get_brain_region_hierarchy()
            .get_parent(&grandchild_id)
            .map(String::as_str),
        Some(parent_id.as_str()),
        "Wave must remain a child of Look for people"
    );
}
