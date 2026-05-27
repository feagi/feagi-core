
extern crate self as feagi_logging_and_errors; // TODO get rid of this
pub use feagi_data::feagi_logging_and_errors::{FeagiErrorKeyTrait, FeagiErrorTrait};

mod spatial_genome;

pub mod feagi_genome_definition_error;
pub mod cortical_area_properties;


pub use spatial_genome::{GenomeCoordAxis, GenomeCoordinate};

