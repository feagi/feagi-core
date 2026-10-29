

pub mod cortical_models;

pub use feagi_connectome_neural::neural_basis::quantization_levels;

pub mod cortical_area {
    pub use feagi_connectome_neural::neural_basis::cortical_area::data::neuron_layout;
    pub use feagi_connectome_neural::neural_basis::cortical_area::cortical_dynamics_model::CorticalDynamicsModel;
    pub use feagi_connectome_neural::neural_basis::cortical_area::cortical_dynamics_quantization::CorticalDynamicsQuantization;
    pub use feagi_connectome_neural::neural_basis::cortical_area::generators::cortical_generator_definition::CorticalGeneratorDefinition;
    
}




