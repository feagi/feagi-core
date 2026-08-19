// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
Corticogenesis: translating a loaded genome into connectome requests.

This is the development path that targets the current core. It takes the `RuntimeGenome`
produced by `feagi-evolutionary` and emits the [`ConnectomeRequest`] sequence that realises it
in an NPU, leaving the actual engine mutation to the caller. Keeping the translation free of
any engine handle is what lets the same code serve the server, tests, and tooling.

Only cortical area creation is translated today. Mapping entries are counted and reported
rather than emitted, because the synapse writer the engine expects
(`CorticalMappingEntryWriterByModelQuant`) is built from a doublet iterator that morphology
evaluation does not yet produce against the refactored core. Reporting them keeps a genome
with mappings from loading as though it were fully realised.
*/

use core::marker::PhantomData;

use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::{
    FeagiIndexQuantization, FeagiIndexQuantizationGenomic,
};
use feagi_data::values::quantizable::QuantizedUnsignedIntegerTrait;
use feagi_evolutionary::runtime::RuntimeGenome;
use feagi_genomic_data::cortical_area_prev::CorticalArea;
use feagi_models::connectome_requests::connectome_request::ConnectomeRequest;
use feagi_models::neuron_model::neuron_model_implementations::feagi_advanced::composers::FeagiAdvancedModelCorticalWriter;
use feagi_models::neuron_model::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;

use crate::types::{BduError, BduResult};

/// Neuron index quantization of the genomic-level engine the NPU is currently fixed to.
type NeuronQuant = <FeagiIndexQuantizationGenomic as FeagiIndexQuantization>::NeuronIndexQuant;

/// Genome property holding the number of neurons packed into each voxel, which becomes the
/// fourth axis of the engine's cortical area dimensions.
const NEURONS_PER_VOXEL_KEY: &str = "neurons_per_voxel";

/// Genome property holding this area's outbound mappings, keyed by destination cortical ID.
const MAPPING_DST_KEY: &str = "cortical_mapping_dst";

/// What corticogenesis did, and what it could not do.
///
/// `mappings_deferred` is not an error: the genome is loaded and its areas are real, but no
/// synapses exist between them, so nothing propagates. Callers should surface this rather than
/// treat the load as complete.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorticogenesisReport {
    /// Number of `CorticalAreaAdd` requests emitted.
    pub areas_added: usize,
    /// Total neurons across every emitted area.
    pub neurons_added: u64,
    /// Mapping entries present in the genome that were not translated.
    pub mappings_deferred: usize,
}

/// Translates a genome into the requests that realise it in an NPU.
///
/// Requests are returned in dependency order: every area is created before any mapping that
/// references it. The caller submits them to the engine in the order given.
pub fn develop_connectome_requests(
    genome: &RuntimeGenome,
) -> BduResult<(Vec<ConnectomeRequest>, CorticogenesisReport)> {
    let mut requests = Vec::with_capacity(genome.cortical_areas.len());
    let mut report = CorticogenesisReport::default();

    // Iteration order of the genome's map is arbitrary, so sort to keep engine indices, and
    // therefore anything derived from them, reproducible across loads.
    let mut areas: Vec<&CorticalArea> = genome.cortical_areas.values().collect();
    areas.sort_by_key(|area| area.cortical_id.as_bytes());

    for area in areas {
        let dimensions = engine_dimensions(area)?;

        let writer = FeagiAdvancedModelCorticalWriter::DefaultNewDimensional {
            dimensions,
            _p: PhantomData::<FeagiAdvancedModelStandardQuant>,
        };

        requests.push(ConnectomeRequest::CorticalAreaAdd {
            TEMP_adding_id: area.cortical_id,
            writer: writer.into(),
        });

        report.areas_added += 1;
        report.neurons_added += neuron_count(area)?;
        report.mappings_deferred += mapping_count(area);
    }

    Ok((requests, report))
}

/// Builds the engine's 4D dimensions for an area: the genome's voxel extents plus the
/// per-voxel neuron count as the fourth axis.
fn engine_dimensions(
    area: &CorticalArea,
) -> BduResult<DimensionalCorticalArea4DDimensions<NeuronQuant>> {
    let (x, y, z, d) = axis_lengths(area)?;

    DimensionalCorticalArea4DDimensions::<NeuronQuant>::try_new_from_usizes(x, y, z, d).map_err(
        |_| {
            BduError::InvalidArea(format!(
                "cortical area '{}' dimensions {x}x{y}x{z}x{d} are not representable in the engine's index quantization",
                area.name
            ))
        },
    )
}

/// Total neurons in an area, which the engine allocates up front.
fn neuron_count(area: &CorticalArea) -> BduResult<u64> {
    let (x, y, z, d) = axis_lengths(area)?;

    (x as u64)
        .checked_mul(y as u64)
        .and_then(|v| v.checked_mul(z as u64))
        .and_then(|v| v.checked_mul(d as u64))
        .ok_or_else(|| {
            BduError::InvalidArea(format!(
                "cortical area '{}' neuron count {x}x{y}x{z}x{d} overflows",
                area.name
            ))
        })
}

/// The four axis lengths of an area, validated to be non-empty.
///
/// The per-voxel neuron count is required rather than defaulted: an area's neuron budget is a
/// genome-level decision, and silently assuming one would change the brain being simulated.
fn axis_lengths(area: &CorticalArea) -> BduResult<(usize, usize, usize, usize)> {
    let x = (*area.dimensions.get_x().as_ref()).quant_to_usize();
    let y = (*area.dimensions.get_y().as_ref()).quant_to_usize();
    let z = (*area.dimensions.get_z().as_ref()).quant_to_usize();

    let d = area
        .properties
        .get(NEURONS_PER_VOXEL_KEY)
        .and_then(|value| value.as_u64())
        .ok_or_else(|| {
            BduError::InvalidArea(format!(
                "cortical area '{}' is missing the '{NEURONS_PER_VOXEL_KEY}' property",
                area.name
            ))
        })? as usize;

    if x == 0 || y == 0 || z == 0 || d == 0 {
        return Err(BduError::InvalidArea(format!(
            "cortical area '{}' has an empty axis in dimensions {x}x{y}x{z}x{d}",
            area.name
        )));
    }

    Ok((x, y, z, d))
}

/// Number of outbound mappings declared by an area.
fn mapping_count(area: &CorticalArea) -> usize {
    area.properties
        .get(MAPPING_DST_KEY)
        .and_then(|value| value.as_object())
        .map(|destinations| destinations.len())
        .unwrap_or(0)
}
