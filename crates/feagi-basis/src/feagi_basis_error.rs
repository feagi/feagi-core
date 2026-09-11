use feagi_basis_error_logging::prelude::*;
use feagi_genome::feagi_genome_error::FeagiGenomeError;
use feagi_neuron::neuron_error::FeagiNeuronError;
use feagi_quantization::feagi_quantization_value::FeagiQuantizationError;
use crate::generic_collections::feagi_index_organizer_error::FeagiIndexOrganizerError;
use crate::thread_messaging::errors::ThreadMessaging;

/// A Genric error type. Anything using this should be updated with more specific errors
#[derive(FeagiFail)]
pub struct FeagiFailDataEtc {
    context: &'static str,
}

generate_feagi_error! {
    /// The root error for the Basis crate
    FeagiBasisError,
    keys: {
        DataEtc: FeagiFailDataEtc,
    },
    sub_errors: {
        IndexOrganizer: FeagiIndexOrganizerError,
        ThreadMessaging: ThreadMessaging,
        Genome: FeagiGenomeError,
        Neuron: FeagiNeuronError,
        Quantization: FeagiQuantizationError
    },
}

