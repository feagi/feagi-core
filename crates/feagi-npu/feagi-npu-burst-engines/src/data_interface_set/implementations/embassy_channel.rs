use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub struct EmbassyDataInterfaceSet<FIQ: FeagiIndexQuantization>
{
    _p: FIQ
}