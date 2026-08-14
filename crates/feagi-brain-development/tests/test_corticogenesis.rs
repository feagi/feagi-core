// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Corticogenesis translation from a loaded genome to connectome requests.

These run against the real genome loader in `feagi-evolutionary` rather than a hand-built
`RuntimeGenome`, so they also cover the flat-to-hierarchical conversion and schema migration
that a genome file goes through before corticogenesis ever sees it.
*/

use feagi_brain_development::corticogenesis::develop_connectome_requests;
use feagi_evolutionary::load_genome_from_json;
use feagi_models::connectome_requests::connectome_request::ConnectomeRequest;

/// The smallest genome FEAGI ships: two 1x1x1 areas (Brain_Power, Brain_Death) and no mappings.
///
/// Read from the shared genome collection rather than a local copy so this cannot drift from the
/// genome the rest of the workspace loads.
fn barebones_genome() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("feagi-evolutionary")
        .join("genomes")
        .join("barebones_genome.json");

    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

#[test]
fn barebones_genome_yields_one_request_per_cortical_area() {
    let genome = load_genome_from_json(&barebones_genome()).expect("barebones genome should load");

    let (requests, report) = develop_connectome_requests(&genome).expect("corticogenesis should succeed");

    assert_eq!(
        report.areas_added,
        genome.cortical_areas.len(),
        "every cortical area in the genome should produce a request"
    );
    assert_eq!(requests.len(), report.areas_added);
    assert!(
        requests
            .iter()
            .all(|request| matches!(request, ConnectomeRequest::CorticalAreaAdd { .. })),
        "barebones declares no mappings, so only area additions should be emitted"
    );
}

#[test]
fn barebones_genome_areas_are_single_neuron() {
    let genome = load_genome_from_json(&barebones_genome()).expect("barebones genome should load");

    let (_requests, report) = develop_connectome_requests(&genome).expect("corticogenesis should succeed");

    // Each area is 1x1x1 voxels holding one neuron, so the neuron total equals the area count.
    assert_eq!(report.neurons_added, report.areas_added as u64);
    assert_eq!(report.mappings_deferred, 0, "barebones has an empty dstmap on every area");
}


#[test]
fn area_missing_neurons_per_voxel_is_rejected() {
    let mut genome = load_genome_from_json(&barebones_genome()).expect("barebones genome should load");

    let victim = *genome.cortical_areas.keys().next().expect("barebones has areas");
    genome
        .cortical_areas
        .get_mut(&victim)
        .expect("area just looked up")
        .properties
        .remove("neurons_per_voxel");

    let error = match develop_connectome_requests(&genome) {
        Err(error) => error,
        Ok(_) => panic!("an area without a neuron budget must not be silently defaulted"),
    };
    assert!(
        error.to_string().contains("neurons_per_voxel"),
        "error should name the missing property, got: {error}"
    );
}

