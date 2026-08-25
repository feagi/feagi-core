use feagi_logging_and_errors::{generate_feagi_error, FeagiError, FeagiErrorKey};

#[derive(FeagiErrorKey)]
pub struct FeagiNeuronInvalidNeuronIndex {
    context: &'static str,
}

generate_feagi_error! {
    FeagiVoxelError,
    keys: {
        InvalidIndex: FeagiNeuronInvalidNeuronIndex
    },
    sub_errors: {

    },
}
