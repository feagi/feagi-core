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
use feagi_data_structures::genomic::brain_regions::{BrainRegion, RegionID, RegionType};
use feagi_data_structures::genomic::cortical_area::CorticalID;

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
    let root_id = root_region.region_id.clone();
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
// Test 2: Subregion can reference parent by UUID
// ═══════════════════════════════════════════════════════════

#[test]
fn test_subregion_parent_uuid_reference() {
    let mut hierarchy = BrainRegionHierarchy::new();

    // Create root region
    let root_region = create_root_region();
    let root_id = root_region.region_id.clone();
    let root_id_str = root_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    // Create subregion
    let subregion = create_subregion();
    let subregion_id = subregion.region_id.clone();
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
    let root_id = root_region.region_id.clone();
    let root_id_str = root_id.to_string();

    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root region");

    // Create subregion
    let subregion = create_subregion();
    let subregion_id = subregion.region_id.clone();
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
    let root_id = root_region.region_id.clone();
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
            .expect(&format!("Failed to add subregion {}", i));

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
    let root_id = root_region.region_id.clone();
    let root_id_str = root_id.to_string();
    hierarchy
        .add_region(root_region, None)
        .expect("Failed to add root");

    // Create child
    let child_region = create_subregion();
    let child_id = child_region.region_id.clone();
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
    let root_id = root_region.region_id.clone();
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
    let root_id = root_region.region_id.clone();
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
