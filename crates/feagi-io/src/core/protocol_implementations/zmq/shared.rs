use crate::FeagiNetworkError;

/// URL endpoint struct for ZMQ endpoints. Has validation checking
#[derive(Debug, Clone, PartialEq)]
pub struct ZmqUrl {
    url: String,
}

impl ZmqUrl {
    pub fn new(url: &String) -> Result<Self, FeagiNetworkError> {
        validate_zmq_url(url)?;
        Ok(ZmqUrl { url: url.to_string() })
    }
}

impl std::fmt::Display for ZmqUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}


fn validate_zmq_url(_url: &String) -> Result<(), FeagiNetworkError> {
    // TODO: inspect url for validity for ZMQ
    Ok(())
}
