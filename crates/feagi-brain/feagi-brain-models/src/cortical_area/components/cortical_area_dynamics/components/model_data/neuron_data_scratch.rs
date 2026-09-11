use crate::cortical_area::common_structs::inverse_outgoing_connection_count::InverseOutgoingConnectionCount;
use crate::cortical_area::components::cortical_area_dynamics::components::cortical_area_quantization::CorticalAreaQuantization;

/// Per neuron data that is not saved to the connectome (regernerated every load). Is required
/// to keep the inverse outgoing connection count
pub trait NeuronDataScratch<CAQ: CorticalAreaQuantization> {
    fn get_inverse_number_mappings_out(&self) -> &InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant>;

    fn set_inverse_number_mappings_out(&mut self, inverse_count: InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant>);
}

/// Represents no data in this field. Is empty/ Unused. No memory will be allocated
pub struct NullNeuronDataScratch<CAQ: CorticalAreaQuantization>(InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant>);

impl<CAQ: CorticalAreaQuantization> NeuronDataScratch<CAQ> for NullNeuronDataScratch<CAQ> {
    fn get_inverse_number_mappings_out(&self) -> &InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant> {
        &self.0
    }

    fn set_inverse_number_mappings_out(&mut self, inverse_count: InverseOutgoingConnectionCount<CAQ::MembranePotentialQuant>) {
        self.0 = inverse_count 
    }
}
