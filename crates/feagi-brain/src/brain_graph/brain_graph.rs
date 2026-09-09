use feagi_data::feagi_data_neuron::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

/// The base trait for describing genomic and or connectome data. Various implementations may keep
/// different data aspects and may be suited for different use cases
pub trait BrainGraph<FIQ: FeagiIndexQuantization> {
    type BrainIDLookup;
    type CorticalIDLookup;
    type MappingIDLookup;
    
    
}