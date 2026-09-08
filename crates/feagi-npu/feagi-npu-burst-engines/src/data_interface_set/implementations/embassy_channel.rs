use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct EmbassyDataInterfaceSet<FIQ: FeagiIndexQuantization>
{
    _p: FIQ
}