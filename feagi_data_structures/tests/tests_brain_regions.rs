//! Tests for the brain_regions module
//!
//! This module contains comprehensive tests for brain region data structures
//! including RegionID, BrainRegion, RegionType, and BrainRegionProperties.

use feagi_data_structures::genomic::brain_regions::*;
use feagi_data_structures::genomic::cortical_area::CoreCorticalType;
use feagi_data_structures::genomic::descriptors::{GenomeCoordinate2D, GenomeCoordinate3D};

#[cfg(test)]
mod test_region_id {
    use super::*;

    #[test]
    fn test_new_region_id() {
        let id1 = RegionID::new();
        let id2 = RegionID::new();
        
        // New IDs should be different
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let region_id = RegionID::from_uuid(uuid);
        assert_eq!(region_id.as_uuid(), uuid);
    }

    #[test]
    fn test_from_string_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let region_id = RegionID::from_string(uuid_str).unwrap();
        assert_eq!(region_id.to_string(), uuid_str);
    }

    #[test]
    fn test_from_string_invalid() {
        let result = RegionID::from_string("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization() {
        let region_id = RegionID::from_string("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let json = serde_json::to_string(&region_id).unwrap();
        assert_eq!(json, "\"550e8400-e29b-41d4-a716-446655440000\"");
    }

    #[test]
    fn test_deserialization() {
        let json = "\"550e8400-e29b-41d4-a716-446655440000\"";
        let region_id: RegionID = serde_json::from_str(json).unwrap();
        assert_eq!(region_id.to_string(), "550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_round_trip() {
        let original = RegionID::new();
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: RegionID = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_from_str() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let region_id: RegionID = uuid_str.parse().unwrap();
        assert_eq!(region_id.to_string(), uuid_str);
    }

    #[test]
    fn test_display() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let region_id = RegionID::from_string(uuid_str).unwrap();
        assert_eq!(format!("{}", region_id), uuid_str);
    }

    #[test]
    fn test_default() {
        let region_id = RegionID::default();
        // Default should generate a new UUID
        assert_ne!(region_id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_conversion_to_uuid() {
        let region_id = RegionID::new();
        let uuid: uuid::Uuid = region_id.into();
        let region_id2 = RegionID::from(uuid);
        assert_eq!(region_id, region_id2);
    }

    #[test]
    fn test_as_bytes() {
        let uuid = uuid::Uuid::from_bytes([0; 16]);
        let region_id = RegionID::from_uuid(uuid);
        assert_eq!(region_id.as_bytes(), &[0; 16]);
    }
}

#[cfg(test)]
mod test_brain_region {
    use super::*;

    #[test]
    fn test_brain_region_creation() {
        let region_id = RegionID::new();
        let region = BrainRegion::new(
            region_id,
            "Visual Cortex".to_string(),
            RegionType::Undefined,
        )
        .unwrap();

        assert_eq!(region.region_id, region_id);
        assert_eq!(region.name, "Visual Cortex");
        assert_eq!(region.region_type, RegionType::Undefined);
        assert_eq!(region.area_count(), 0);
    }

    #[test]
    fn test_add_remove_areas() {
        let region_id = RegionID::new();
        let mut region = BrainRegion::new(
            region_id,
            "Test".to_string(),
            RegionType::Undefined,
        )
        .unwrap();

        // Create test cortical IDs using core types
        let area1 = CoreCorticalType::Power.to_cortical_id();
        let area2 = CoreCorticalType::Death.to_cortical_id();

        // Add areas
        assert!(region.add_area(area1));
        assert!(region.add_area(area2));
        assert!(!region.add_area(area1)); // Already exists

        assert_eq!(region.area_count(), 2);
        assert!(region.contains_area(&area1));
        assert!(region.contains_area(&area2));

        // Remove area
        assert!(region.remove_area(&area1));
        assert!(!region.remove_area(&area1)); // Already removed

        assert_eq!(region.area_count(), 1);
        assert!(!region.contains_area(&area1));
        assert!(region.contains_area(&area2));
    }

    #[test]
    fn test_with_areas() {
        use feagi_data_structures::genomic::cortical_area::CorticalID;
        
        let area1 = CoreCorticalType::Power.to_cortical_id();
        let area2 = CoreCorticalType::Death.to_cortical_id();
        // Create a third area by building a custom byte array
        let area3 = CorticalID::try_from_bytes(b"___test1").unwrap();
        
        let region_id = RegionID::new();
        let region = BrainRegion::new(
            region_id,
            "Test".to_string(),
            RegionType::Undefined,
        )
        .unwrap()
        .with_areas(vec![area1, area2, area3]);

        assert_eq!(region.area_count(), 3);
        assert!(region.contains_area(&area1));
        assert!(region.contains_area(&area2));
        assert!(region.contains_area(&area3));
    }

    #[test]
    fn test_properties() {
        let region_id = RegionID::new();
        let mut region = BrainRegion::new(
            region_id,
            "Test".to_string(),
            RegionType::Undefined,
        )
        .unwrap();

        // Test setting various properties using HashMap
        region.add_property("description".to_string(), serde_json::json!("Visual processing region"));
        region.add_property("coordinate_2d".to_string(), serde_json::json!([10, 20]));
        region.add_property("coordinate_3d".to_string(), serde_json::json!([10, 20, 30]));

        assert_eq!(region.get_property("description"), Some(&serde_json::json!("Visual processing region")));
        assert!(region.get_property("coordinate_2d").is_some());
        assert!(region.get_property("coordinate_3d").is_some());
    }

    #[test]
    fn test_clear_areas() {
        let area1 = CoreCorticalType::Power.to_cortical_id();
        let area2 = CoreCorticalType::Death.to_cortical_id();
        
        let region_id = RegionID::new();
        let mut region = BrainRegion::new(
            region_id,
            "Test".to_string(),
            RegionType::Undefined,
        )
        .unwrap()
        .with_areas(vec![area1, area2]);

        assert_eq!(region.area_count(), 2);

        region.clear_areas();
        assert_eq!(region.area_count(), 0);
    }

    #[test]
    fn test_serialization() {
        let area1 = CoreCorticalType::Power.to_cortical_id();
        
        let region_id = RegionID::new();
        let region = BrainRegion::new(
            region_id,
            "Test Region".to_string(),
            RegionType::Undefined,
        )
        .unwrap()
        .with_areas(vec![area1]);

        // Serialize to JSON
        let json = serde_json::to_string(&region).unwrap();

        // Deserialize back
        let deserialized: BrainRegion = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.region_id, region_id);
        assert_eq!(deserialized.name, "Test Region");
        assert_eq!(deserialized.region_type, RegionType::Undefined);
        assert!(deserialized.contains_area(&area1));
    }

    #[test]
    fn test_genome_format_serialization() {
        let area1 = CoreCorticalType::Power.to_cortical_id();
        let area2 = CoreCorticalType::Death.to_cortical_id();
        
        let region_id = RegionID::new();
        let mut region = BrainRegion::new(
            region_id,
            "Root Brain Region".to_string(),
            RegionType::Undefined,
        )
        .unwrap()
        .with_areas(vec![area1, area2]);

        // Set properties similar to genome format using HashMap
        region.add_property("description".to_string(), serde_json::json!("Default root region for brain organization"));
        region.add_property("coordinate_2d".to_string(), serde_json::json!([0, 0]));
        region.add_property("coordinate_3d".to_string(), serde_json::json!([0, 0, 0]));
        region.add_property("signature".to_string(), serde_json::json!(""));

        // Serialize to JSON (pretty print for inspection)
        let json = serde_json::to_string_pretty(&region).unwrap();

        // Deserialize back
        let deserialized: BrainRegion = serde_json::from_str(&json).unwrap();

        // Verify all fields
        assert_eq!(deserialized.region_id, region_id);
        assert_eq!(deserialized.name, "Root Brain Region");
        assert_eq!(deserialized.get_property("description"), Some(&serde_json::json!("Default root region for brain organization")));
        assert!(deserialized.get_property("coordinate_2d").is_some());
        assert!(deserialized.get_property("coordinate_3d").is_some());
        assert_eq!(deserialized.get_property("signature"), Some(&serde_json::json!("")));
    }
}

