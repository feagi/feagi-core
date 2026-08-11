use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::burst_engine::burst_engine::BurstEngine;
use crate::burst_engine_enum::ComposableBurstEngineEnum;
// TODO consolidate some functions to lower the number of calls

/// An extension to the burst engine that allows editing the connectome between bursts
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ>
{
    fn add_cortical_areas(&mut self);

    fn remove_cortical_areas(&mut self);

    fn overwrite_neuron_firings(&mut self);

    fn add_cortical_mappings(&mut self);

    fn remove_cortical_mappings(&mut self);
}


