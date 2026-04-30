// TODO some neuron properties are specific to specific neuron models, This folder will hold traits and implementations

mod dimensional_neuron_data_traits;
mod dimensional_cortical_configuration_traits;
mod dimensional_cortical_area_generator_traits;

pub(crate) mod feagi_standard;


pub(crate) use dimensional_cortical_configuration_traits::DimensionalCorticalConfigurationTrait;
pub(crate) use dimensional_neuron_data_traits::*;
pub use dimensional_cortical_area_generator_traits::DimensionalCorticalAreaGeneratorTrait;
