use feagi_data::quantization_levels::membrane_potential_quantization::MembranePotentialQuantization;



pub struct CorticalIndexesDendrite {
    local_neuron_start: u32,
    local_neuron_length: u32, // >> 4 for bitbatch length
    activity_bitbatch_start: u32,// or less
    flags: u8,
    quants: u8
}

pub struct CorticalIndexesBody {
    local_neuron_start: u32,
    local_neuron_length: u32, // >> 4 for bitbatch length
    activity_bitbatch_start: u32,// or less
    flags: u8,
    quants: u8
}


pub struct CorticalIndexesAxon {
    local_neuron_start: u32,

    flags: u8,
    quants: u8
}