use rayon::prelude::*;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::engines::rayon::data::RayonEngineData;

fn process_synapses<FIQ: FeagiIndexQuantization>(data: &RayonEngineData<FIQ>)
{
    let burst_index = data.burst_index;

    // We access `data` through a shared `&` and mutate disjoint slots via the
    // `get_mut_par` accessors
    unsafe {

        // no clustering with synapses, since the way they access data *may* be sporadic.




    }




}