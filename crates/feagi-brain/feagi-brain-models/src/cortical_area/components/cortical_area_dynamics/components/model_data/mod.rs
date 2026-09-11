
pub use cortical_data_properties::{CorticalDataProperties};
pub use cortical_data_internal::{NullCorticalDataInternal, CorticalDataInternal};
pub use cortical_data_scratch::{NullCorticalDataScratch, CorticalDataScratch};
pub use neuron_data_internal::{NullNeuronDataInternal, NeuronDataInternal};
pub use neuron_data_scratch::{NullNeuronDataScratch, NeuronDataScratch};

mod cortical_data_properties;
mod cortical_data_internal;
mod cortical_data_scratch;
mod neuron_data_internal;
mod neuron_data_scratch;