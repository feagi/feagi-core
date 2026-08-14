use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::npu::burst_engine::burst_engine::BurstEngine;
// TODO consolidate some functions to lower the number of calls

/// An extension to the burst engine that allows editing the connectome between bursts
pub trait ComposableBurstEngine<FIQ: FeagiIndexQuantization>: BurstEngine<FIQ>
{
    async fn edit_connectome(&mut self);
}


