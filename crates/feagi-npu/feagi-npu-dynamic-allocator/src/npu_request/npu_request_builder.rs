use core::marker::PhantomData;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron::npu_requests::add_cortical_area::NPURequestCorticalArea;

/// Endpoint emitted from an NPU that lets you construct requests to edit the connectome of the
/// NPU with strong compile time typed checking. Rust FTW. Can only be constructed by the NPU
/// so that the context is correct
#[derive(Debug, Clone, PartialEq)]
pub struct NPURequestBuilder<FIQ: FeagiIndexQuantization> {
    _p: PhantomData<FIQ>,
}

impl<FIQ: FeagiIndexQuantization> NPURequestBuilder<FIQ> {
    
    /// Cortical area related request
    pub fn cortical_area() -> NPURequestCorticalArea<FIQ> {
        NPURequestCorticalArea::create_npu_cortical_request()
    }
}