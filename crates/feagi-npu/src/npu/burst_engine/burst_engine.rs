use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// Functions that all burst engines have. These are related to execution
pub trait BurstEngine<FIQ: FeagiIndexQuantization> {

    /// Run a number of bursts before stopping
    async fn run_kernel(&mut self);

    // TODO data sync
}


