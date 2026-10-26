//! Brain regions: hierarchical groupings of cortical areas (and child regions), with their
//! layout coordinates and IO designations. Converted from the old `region` endpoints.

use feagi_basis::feagi_basis_proc_common::define_request_message_category;
use std::collections::HashMap;
use feagi_cns_structures::basis::cortical_area::cortical_id::Base64EncodedCorticalID;

/// Summary of one brain region and its members, as listed by `RegionsMembers`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegionMembersEntry {
    /// Display name of the region
    pub title: String,
    /// Region description (currently always empty)
    pub description: String,
    /// ID of the parent region, None for the root region
    pub parent_region_id: Option<String>,
    /// 2D layout coordinate
    pub coordinate_2d: [i32; 2],
    /// 3D layout coordinate
    pub coordinate_3d: [i32; 3],
    /// IDs of member cortical areas
    pub areas: Vec<Base64EncodedCorticalID>,
    /// IDs of child regions
    pub regions: Vec<String>,
    /// Cortical area IDs acting as region inputs
    pub inputs: Vec<String>,
    /// Cortical area IDs acting as region outputs
    pub outputs: Vec<String>,
    /// Cortical area IDs explicitly designated as region inputs
    pub designated_inputs: Vec<String>,
    /// Cortical area IDs explicitly designated as region outputs
    pub designated_outputs: Vec<String>,
}

/// New coordinates for one region, used by `RelocateMembers`
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct RegionRelocation {
    /// New 2D layout coordinate
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "coordinates_2d")]
    pub coordinate_2d: Option<[i32; 2]>,
    /// New 3D layout coordinate
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "coordinates_3d")]
    pub coordinate_3d: Option<[i32; 3]>,
}

define_request_message_category! {
    category_name: "Region",
    base_path: "v1/region",
    category_description: "Brain regions: hierarchical groupings of cortical areas and child regions",
    read: {
        "RegionsMembers": {
            path: "regions_members",
            path_item_descriptions: {
                regions: "Every brain region, keyed by region ID",
            },
            description: "Get all brain regions with their member cortical areas and child regions",
            alloc_only: true,
            response: {
                regions: HashMap<String, RegionMembersEntry>,
            },
        },
        "Regions": {
            path: "regions",
            description: "Get the IDs of all brain regions",
            alloc_only: true,
            response: {
                region_ids: Vec<String>,
            },
        },
        "RegionTitles": {
            path: "region_titles",
            description: "Get a mapping of brain region IDs to titles",
            alloc_only: true,
            response: {
                titles: HashMap<String, String>, "Region ID to title",
            },
        },
        "Region": {
            path: "region/:region_id<String>",
            path_item_descriptions: {
                region_id: "ID of the brain region",
            },
            description: "Get detailed properties of a brain region",
            alloc_only: true,
            response: {
                title: String, "Display name",
                description: String,
                coordinate_2d: [i32; 2], "2D layout coordinate",
                coordinate_3d: [i32; 3], "3D layout coordinate",
                areas: Vec<String>, "IDs of member cortical areas",
                regions: Vec<String>, "IDs of child regions",
                parent_region_id: Option<String>, "ID of the parent region, None for the root region",
            },
            error: {
                RegionNotFound: "No brain region has that ID",
            },
        },
    },
    create: {
        "Region": {
            path: "region",
            description: "Create a new brain region",
            alloc_only: true,
            payload: {
                title: String, "Display name (alias: name)",
                region_id: Option<String>, "ID for the new region; generated if omitted",
                parent_region_id: Option<String>, "ID of the parent region",
                region_type: Option<String>, "Functional type; defaults to Undefined",
                coordinate_2d: [i32; 2], "2D layout coordinate (alias: coordinates_2d)",
                coordinate_3d: [i32; 3], "3D layout coordinate (alias: coordinates_3d)",
                areas: Option<Vec<Base64EncodedCorticalID>>, "IDs of cortical areas to place in the region",
                regions: Option<Vec<String>>, "IDs of regions to place in the region",
            },
            response: {
                region_id: String, "ID of the created region",
                title: String,
                parent_region_id: Option<String>,
                coordinate_2d: [i32; 2],
                coordinate_3d: [i32; 3],
                areas: Vec<String>,
                regions: Vec<String>,
                inputs: Option<Vec<Base64EncodedCorticalID>>, "Cortical area IDs acting as region inputs",
                outputs: Option<Vec<Base64EncodedCorticalID>>, "Cortical area IDs acting as region outputs",
            },
            error: {
                InvalidRegionId: "The given region_id is not a valid region ID",
            },
        },
        "Clone": {
            path: "clone",
            description: "Clone a non-root brain region; queues an amalgamation of the exported region",
            alloc_only: true,
            payload: {
                source_region_id: String, "ID of the region to clone (alias: region_id)",
                region_name: String, "Title for the clone (alias: title)",
            },
            response: {
                message: String,
                amalgamation_id: String, "ID of the queued amalgamation",
                circuit_size: [i32; 3], "Size of the cloned circuit",
            },
            error: {
                RegionNotFound: "No brain region has that ID",
                RootRegionNotCloneable: "The root region cannot be cloned",
                AmalgamationAlreadyPending: "An amalgamation is already pending; cancel it first",
            },
        },
    },
    edit: {
        "Region": {
            path: "region",
            description: "Update properties of a brain region",
            alloc_only: true,
            payload: {
                region_id: String, "ID of the region to update",
                properties: HashMap<String, serde_json::Value>, "Properties to update, by name",
            },
            response: {
                message: String,
                region_id: String,
            },
        },
        "RelocateMembers": {
            path: "relocate_members",
            description: "Update the layout coordinates of one or more brain regions",
            alloc_only: true,
            payload: {
                regions: HashMap<String, RegionRelocation>, "New coordinates, keyed by region ID",
            },
            response: {
                message: String,
                region_ids: String, "Comma separated IDs of the updated regions",
            },
        },
        "ChangeRegionParent": {
            path: "change_region_parent",
            description: "Change the parent of a brain region",
            alloc_only: true,
            response: {
                message: String,
            },
        },
        "ChangeCorticalAreaRegion": {
            path: "change_cortical_area_region",
            description: "Change the region association of a cortical area",
            alloc_only: true,
            response: {
                message: String,
            },
        },
    },
    delete: {
        "Region": {
            path: "region",
            description: "Delete a brain region",
            alloc_only: true,
            payload: {
                region_id: String, "ID of the region to delete (alias: id)",
            },
            response: {
                message: String,
                region_id: String,
            },
        },
        "RegionAndMembers": {
            path: "region_and_members",
            description: "Delete a brain region together with its members",
        },
    },
}

// NOTES:
// RegionsMembers: old response was a bare map of region ID -> object; wrapped in `regions`.
// Regions / RegionTitles: old responses were a bare array / bare map; wrapped in `region_ids` / `titles`.
// Coordinates typed as i32 arrays (guessed; old code passed raw JSON arrays through).
// create Region: old code also accepted arbitrary extra keys, stored as region properties; not modelled.
// edit Region: old payload was flat (region_id + arbitrary property keys); extra keys moved into `properties` (untyped Values).
// RelocateMembers: old payload was a bare map region ID -> coordinates; wrapped in `regions`.
// RelocateMembers: setting parent_region_id was explicitly rejected; not modelled.
// DELETE region_and_members: unimplemented stub, payload and response fields unknown.
// PUT change_region_parent: unimplemented stub, payload unknown; response only had `message`.
// PUT change_cortical_area_region: unimplemented stub, payload unknown; response only had `message`.
