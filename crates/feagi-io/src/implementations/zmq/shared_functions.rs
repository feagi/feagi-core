use crate::FeagiNetworkError;

pub fn validate_zmq_url(_url: &String) -> Result<(), FeagiNetworkError> {
    // TODO: inspect url for validity for ZMQ
    Ok(())
}
