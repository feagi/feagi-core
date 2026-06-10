use feagi_structures::feagi_data::create_quantized_index_count_wrapper;
use feagi_structures::feagi_data::quantizable_linear::base_types::QuantizedIndexCountTrait;


create_quantized_index_count_wrapper!(NPUGlobalBurstCounter);

impl<QE: QuantizedIndexCountTrait> NPUGlobalBurstCounter<QE> {

    /// Increments by one. Safely overflows to 0 when reaching the max of its quantized value.
    /// When this happens, this function will return true. Every other time, it will return false
    pub fn increment_burst_count_with_rollover(&mut self) -> bool {
         if self.0 == QE::QUANT_MAX {
             // about to roll over
             self.0 = QE::QUANT_ZERO;
             return true;
         }
        self.0 += QE::QUANT_ONE;
        false
    }
}




