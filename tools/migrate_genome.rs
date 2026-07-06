// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Genome Migration Tool

Migrates old-format genomes (v2.1 with non-compliant cortical_area IDs) to new format.

Usage:
  cd feagi-core && cargo run --bin migrate_genome -- <input_genome.json> <output_genome.json>

Example:
  cd feagi-core && cargo run --bin migrate_genome -- ../brain-visualizer/godot_source/Resources/genomes/essential_genome.json essential_genome_v3.json

Copyright 2025 Neuraville Inc.
Licensed under the Apache License, Version 2.0
*/

use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!(
            "Usage: {} <input_genome.json> <output_genome.json>",
            args[0]
        );
        eprintln!("\nExample:");
        eprintln!(
            "  {} essential_genome.json essential_genome_v3.json",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("🧬 FEAGI Genome Migration Tool");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📂 Input:  {}", input_path);
    println!("📂 Output: {}", output_path);
    println!();

    // Check if input file exists
    if !Path::new(input_path).exists() {
        eprintln!("❌ Error: Input file '{}' not found", input_path);
        std::process::exit(1);
    }

    // Load genome
    println!("📖 Loading genome...");
    let genome_json_str = fs::read_to_string(input_path)?;
    let genome_json: serde_json::Value = serde_json::from_str(&genome_json_str)?;

    // Get genome metadata
    let genome_id = genome_json
        .get("genome_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let version = genome_json
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("   Genome ID: {}", genome_id);
    println!("   Version:   {}", version);
    println!();

    // Perform migration
    println!("🔄 Migrating cortical_area IDs...");
    let migration_result = feagi_evolutionary::migrate_genome(&genome_json)?;

    println!(
        "   ✅ Migrated {} cortical_area IDs",
        migration_result.cortical_ids_migrated
    );
    println!();

    // Show ID mappings
    if !migration_result.id_mapping.is_empty() {
        println!("📋 ID Mappings:");
        let mut mappings: Vec<_> = migration_result.id_mapping.iter().collect();
        mappings.sort_by_key(|(old, _)| *old);

        for (old_id, new_id) in mappings {
            println!("   {} → {}", old_id, new_id);
        }
        println!();
    }

    // Show warnings
    if !migration_result.warnings.is_empty() {
        println!("⚠️  Warnings:");
        for warning in &migration_result.warnings {
            println!("   {}", warning);
        }
        println!();
    }

    // Update version to 2.2 (migrated with new cortical_area IDs)
    let mut migrated_genome = migration_result.genome;
    if let Some(genome_obj) = migrated_genome.as_object_mut() {
        genome_obj.insert("version".to_string(), serde_json::json!("2.2"));

        // Add migration metadata
        let mut description = genome_obj
            .get("genome_description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        description.push_str(" [Migrated to new cortical_area ID format v2.2]");
        genome_obj.insert(
            "genome_description".to_string(),
            serde_json::json!(description),
        );
    }

    // Save migrated genome
    println!("💾 Saving migrated genome...");
    let migrated_json_str = serde_json::to_string_pretty(&migrated_genome)?;
    fs::write(output_path, migrated_json_str)?;

    println!("   ✅ Saved to: {}", output_path);
    println!();

    // Validate migrated genome
    println!("🔍 Validating migrated genome...");
    match feagi_evolutionary::load_genome_from_json(&serde_json::to_string(&migrated_genome)?) {
        Ok(runtime_genome) => {
            let validation_result = feagi_evolutionary::validate_genome(&runtime_genome);

            if validation_result.errors.is_empty() {
                println!("   ✅ Validation passed!");
            } else {
                println!("   ❌ Validation errors:");
                for error in &validation_result.errors {
                    println!("      {}", error);
                }
            }

            if !validation_result.warnings.is_empty() {
                println!("   ⚠️  Validation warnings:");
                for warning in &validation_result.warnings {
                    println!("      {}", warning);
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to load migrated genome: {}", e);
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Migration complete!");

    Ok(())
}
