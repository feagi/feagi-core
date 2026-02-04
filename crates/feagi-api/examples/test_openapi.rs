// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

// Test program to validate OpenAPI spec
use feagi_api::openapi::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let spec = ApiDoc::openapi();

    println!("=== FEAGI OpenAPI Spec Validation ===\n");

    // Count total paths
    let paths = &spec.paths;
    let total_paths = paths.paths.len();

    println!("✓ Total paths registered: {}", total_paths);
    println!("\nPaths:");

    let mut path_list: Vec<_> = paths.paths.keys().collect();
    path_list.sort();

    // Group by tags to see what Swagger will show
    let mut tags_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for path in &path_list {
        let path_item = paths.paths.get(*path).unwrap();
        // Get tags from operations
        for (_method, operation) in &path_item.operations {
            if let Some(tags) = &operation.tags {
                for tag in tags {
                    tags_map
                        .entry(tag.clone())
                        .or_insert_with(Vec::new)
                        .push((*path).clone());
                }
            } else {
                tags_map
                    .entry("untagged".to_string())
                    .or_insert_with(Vec::new)
                    .push((*path).clone());
            }
        }

        // Print skipped in loop
    }

    println!("\n=== Tags (how Swagger groups endpoints) ===");
    let mut tag_list: Vec<_> = tags_map.keys().collect();
    tag_list.sort();
    for tag in tag_list {
        let endpoints = &tags_map[tag];
        println!("\nTag: '{}' ({} endpoints)", tag, endpoints.len());
        for endpoint in endpoints.iter().take(10) {
            println!("  - {}", endpoint);
        }
        if endpoints.len() > 5 {
            println!("  ... and {} more", endpoints.len() - 5);
        }
    }

    // Count schemas
    if let Some(components) = &spec.components {
        let schema_count = components.schemas.len();
        println!("\n✓ Total schemas registered: {}", schema_count);
    }

    println!("\n=== Summary ===");
    println!("Total endpoints: {}", total_paths);
    println!("Expected: 64");

    if total_paths < 64 {
        println!("\n❌ ISSUE: Missing {} endpoints!", 64 - total_paths);
    } else {
        println!("\n✅ All endpoints present!");
    }
}
