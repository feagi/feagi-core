use feagi_data::quantization_levels::feagi_index_quantization::FeagiGlobalQuantization;
use feagi_data::values::quantizable::QuantizedIndexCountTrait;
use crate::engines::blocking::rayon::data::engine_data::RayonEngineData;

/// Contains several methods of processing the neurons in the rayon burst engine
pub enum RayonNeuronProcessing<FGQ: FeagiGlobalQuantization> {
    /// Updates visualizer data always as each loop processes neurons in batches of 8 and directly
    /// writes to the bitpacked u8, without need a separate loop to check for visualization
    VisualizerInline
}

impl<FGQ: FeagiGlobalQuantization> RayonNeuronProcessing<FGQ> {
    
    /// 
    pub fn process_neurons(&self, data: &mut RayonEngineData<FGQ>)
    {
        match self {
            RayonNeuronProcessing::VisualizerInline => {

                // Rayon doesnt need to consolidate FCL, work stealing is sufficient
                
                // for each bit pack u8, start a new mut u8
                // loop 0 - 8 get neuron engine index (start then increment)
                
                // // get mut neuron fcl mp
                // // if fcl is zero, continue loop
                // // get cortical area index
                // // get cortical context
                // // get local neuron index
                // // get mut neuron history
                // // get dimensional layout TODO not always dimensional!
                // // get cortical model data
                // // get mut neuron model data
                // // get mut neuron potential
                // // is_firing = feagi standard model firing
                
                // // neuron fcl mp = 0
                // // update neuron history
                // // update bit 0-8 for if fired
                
                // 
                
                increment_burst_counter(data);
            }
        }
    }
}







fn increment_burst_counter<FGQ: FeagiGlobalQuantization>(data: &mut RayonEngineData<FGQ>)
{
    if *data.burst_index.as_ref() == FGQ::GlobalBurstIndexQuant::QUANT_MAX {
        // OVERFLOW!

        *data.burst_index = FGQ::GlobalBurstIndexQuant::QUANT_MAX / FGQ::GlobalBurstIndexQuant::from_usize(2)
        
        // TODO call the right functions
    }
    else
    {
        *data.burst_index.as_mut() += FGQ::GlobalBurstIndexQuant::QUANT_MAX;
    }
    
}