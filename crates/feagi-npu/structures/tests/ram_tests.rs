
use crate::connectome::ConnectomeAllocRam;


#[cfg(test)]
mod connectome {
    use super::*;

    #[test]
    fn test_ram_npu() {
        let mut connectome = ConnectomeAllocRam::new();
        let neuron_mapper;



        let source_index = connectome.create_dimensional_neuron_cortical_area_with_default_neurons(

        )?;
        let destination_index = connectome.create_dimensional_neuron_cortical_area_with_default_neurons(

        )?;

        connectome.add_nonplastic_connection_from_dimensional_area_to_dimensional_area(

        )?;

        let fire_queue_ref = connectome.burst();

    }
}



