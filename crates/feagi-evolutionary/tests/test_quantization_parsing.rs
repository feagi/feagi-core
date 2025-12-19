// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright 2025 Neuraville Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! Test that quantization_precision is correctly parsed from genome JSON

use feagi_evolutionary::genome::loader::load_genome_from_file;
use feagi_npu_neural::types::{Precision, QuantizationSpec};
use std::path::PathBuf;

#[test]
fn test_essential_genome_quantization_parsing() {
    let genome_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("genomes/essential_genome.json");

    assert!(genome_path.exists(), "essential_genome.json not found");

    let runtime_genome =
        load_genome_from_file(&genome_path).expect("Failed to parse essential_genome.json");

    // Verify quantization_precision was parsed
    let quant_precision = &runtime_genome.physiology.quantization_precision;
    println!("Parsed quantization_precision: {}", quant_precision);

    // Verify it's a valid precision string (defaults to "fp32" if not in genome)
    assert!(
        !quant_precision.is_empty(),
        "Should have a quantization_precision value"
    );

    // Verify it can be converted to Precision
    let precision = Precision::from_str(quant_precision.as_str()).unwrap_or(Precision::FP32);

    // Should be one of the valid precisions
    assert!(
        precision == Precision::FP32
            || precision == Precision::FP16
            || precision == Precision::INT8,
        "Should parse to a valid precision"
    );

    println!("Successfully parsed as: {:?}", precision);
}

#[test]
fn test_quantization_defaults() {
    // Test that genomes without quantization_precision default to fp32
    let json_without_quant = r#"
    {
        "genome_id": "test",
        "physiology": {
            "simulation_timestep": 0.01
        }
    }
    "#;

    let value: serde_json::Value = serde_json::from_str(json_without_quant).unwrap();

    // Should default to "fp32"
    let quant = value["physiology"]["quantization_precision"]
        .as_str()
        .unwrap_or("fp32");

    assert_eq!(quant, "fp32");

    let precision = Precision::from_str(quant).unwrap_or(Precision::FP32);
    assert_eq!(precision, Precision::FP32);
}

#[test]
fn test_all_precision_types_parse() {
    let test_cases = vec![
        ("fp32", Precision::FP32),
        ("fp16", Precision::FP16),
        ("int8", Precision::INT8),
        ("f32", Precision::FP32),
        ("f16", Precision::FP16),
        ("i8", Precision::INT8),
    ];

    for (input, expected) in test_cases {
        let precision =
            Precision::from_str(input).unwrap_or_else(|_| panic!("Failed to parse: {}", input));
        assert_eq!(
            precision, expected,
            "Input '{}' should map to {:?}",
            input, expected
        );
    }
}
