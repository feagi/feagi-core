// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Reading firing activity out of a burst engine for visualization.
//!
//! The engine stores neurons as flat, engine-wide vectors, which is what makes the burst kernels
//! fast but says nothing about where a neuron sits in its cortical area. Visualizers need the
//! opposite view: per area, the coordinates of the neurons that fired. This module walks the
//! engine's per-area index bookkeeping to turn one into the other.
//!
//! The output is deliberately shaped as parallel coordinate and potential vectors rather than a
//! vector of structs, because that is the layout the neuron voxel wire formats consume, so a
//! publisher can hand these straight over without another transposition.

use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_data::values::quantizable::{QuantizedIndexCountTrait, WrappedQuantizedIndexCount};
use feagi_models::wrapped_index_collections::{CorticalEngineIndex, NeuronEngineIndex};

use crate::engines::rayon::rayon_burst_engine::RayonBurstEngine;

/// The neurons that fired in one cortical area during the most recent burst.
///
/// `coords_*` and `potentials` are parallel: index `i` of each describes the same neuron. Where a
/// cortical area holds more than one neuron per voxel, several entries can share the same
/// coordinate, one per firing neuron in that voxel.
#[derive(Debug, Clone, PartialEq)]
pub struct CorticalAreaFireSnapshot<FIQ: FeagiIndexQuantization> {
    /// Which area within the engine this describes.
    pub cortical_index: CorticalEngineIndex<FIQ::CorticalAreaIndexCountQuant>,
    pub coords_x: Vec<u32>,
    pub coords_y: Vec<u32>,
    pub coords_z: Vec<u32>,
    /// Membrane potential of each firing neuron at the end of the burst.
    pub potentials: Vec<f32>,
}

impl<FIQ: FeagiIndexQuantization> CorticalAreaFireSnapshot<FIQ> {
    /// Number of firing neurons captured.
    pub fn len(&self) -> usize {
        self.potentials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.potentials.is_empty()
    }
}

impl<FIQ: FeagiIndexQuantization> RayonBurstEngine<FIQ> {
    /// Collects the neurons that fired in the most recent burst, grouped by cortical area.
    ///
    /// Areas with no firing neurons are omitted rather than returned empty, since a visualizer
    /// only needs to draw what is active and quiet areas dominate in a typical brain.
    pub fn fire_queue_snapshot(&self) -> Vec<CorticalAreaFireSnapshot<FIQ>> {
        let data = self.engine_data();

        let number_areas = data.cortical_neuron_count.as_slice().len();
        let index_lookups = data.cortical_neuron_index_lookup_table.as_slice();
        let cortical_lookups = data.cortical_index_lookup_table.as_slice();
        let layouts = data.cortical_layout_dimensional_data.as_slice();
        let membrane_potentials = data.neuron_membrane_data.mp_f32.as_slice();

        let mut snapshots = Vec::new();

        for area in 0..number_areas {
            let index_lookup = &index_lookups[area];
            let layout_index = cortical_lookups[area].cortical_layout_index.deref().quant_to_usize();
            let dimensions = layouts[layout_index].dimensions;

            let first_neuron = index_lookup.cortical_first_neuron_engine_index.deref().quant_to_usize();

            // The burst packs firing state into a bit per neuron, so the set bits are exactly the
            // neurons to draw. Reading them costs one pass over `neuron_count / 8` bytes with
            // whole zero bytes skipped, rather than a test per neuron.
            let bitmap_index = FIQ::CorticalAreaIndexCountQuant::quant_from_usize(area);
            let Some((firing_bits, _)) = data.neuron_voxel_is_firing.get_slice_by_index(bitmap_index) else {
                continue;
            };

            let firing_count = firing_bits.count_set_bits();
            if firing_count == 0 {
                continue;
            }

            let mut coords_x = Vec::with_capacity(firing_count);
            let mut coords_y = Vec::with_capacity(firing_count);
            let mut coords_z = Vec::with_capacity(firing_count);
            let mut potentials = Vec::with_capacity(firing_count);

            firing_bits.for_each_set_bit(|local| {
                let local_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexQuant> = NeuronCorticalLocalIndex::new(local);
                // The 4th axis is the neuron's index within its voxel, which visualizers do not
                // draw, so only the spatial axes are carried over.
                let coordinate = dimensions.linear_index_to_coordinate_unchecked(local_index);

                coords_x.push(coordinate.get_x().deref().quant_to_usize() as u32);
                coords_y.push(coordinate.get_y().deref().quant_to_usize() as u32);
                coords_z.push(coordinate.get_z().deref().quant_to_usize() as u32);

                let engine_index: NeuronEngineIndex<FIQ::NeuronIndexQuant> =
                    NeuronEngineIndex::quant_from_usize(first_neuron + local.quant_to_usize());
                let mp_slot = index_lookup.get_neuron_mp_index(&engine_index).deref().quant_to_usize();
                potentials.push(membrane_potentials[mp_slot].deref());
            });

            snapshots.push(CorticalAreaFireSnapshot {
                cortical_index: CorticalEngineIndex::quant_from_usize(area),
                coords_x,
                coords_y,
                coords_z,
                potentials,
            });
        }

        snapshots
    }
}
