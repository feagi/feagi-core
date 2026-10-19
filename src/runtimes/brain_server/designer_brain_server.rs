use feagi_interop::feagi_message::feagi_message_path::{FeagiMessagePath, FeagiRequestMessagePath};
use feagi_interop::feagi_message::feagi_request_message::{FeagiRequestMessage, FeagiRequestMessageDescriptor};
use feagi_interop::feagi_message::feagi_request_message::message_components::MessageQueryables;

pub struct DesignerBrainServer {
    
}

fn runtime_loop() {
    
    
    
    
    
}



//region System

pub struct HealthCheckMessageRequest {
    // no fields!
}

impl FeagiRequestMessage for HealthCheckMessageRequest {
    type Descriptor = HealthCheckMessageRequestDescriptor;

    fn compose(
        parameters: Self::Descriptor::Parameters,
        queryables: Self::Descriptor::Queryables,
        payload: Self::Descriptor::RequestPayload) -> Self {

        // no data!
        Self
    }

    fn decompose(self)
        -> (
            Self::Descriptor::Parameters,
            Self::Descriptor::Queryables,
            Self::Descriptor::RequestPayload)
    {
        ((),(),())
    }
}


pub struct HealthCheckMessageRequestDescriptor;

impl FeagiRequestMessageDescriptor for HealthCheckMessageRequestDescriptor {
    const MESSAGE_DESCRIPTION: &'static str = "FEAGI Server System Level Statuses";
    type Message = HealthCheckMessageRequest;
    type MessagePath = HealthCheckMessageRequestPath;
    type Parameters = ();
    type Queryables = ();

    type RequestPayload = ();
    type ResponsePayload = HealthCheckResponse;

    type ResponseError = ();
}

pub struct HealthCheckMessageRequestPath;

impl FeagiMessagePath for HealthCheckMessageRequestPath {
    type TypesTuple = (&'static str, &'static str);
    type ConfigurablesTuple = ();

    fn create_key_expr_for_zenoh(configurables: &Self::ConfigurablesTuple) -> () {
        todo!()
    }
}

impl FeagiRequestMessagePath for HealthCheckMessageRequestPath {
    fn add_path_to_pico_serve_router<Q: MessageQueryables>(configurables: &Self::ConfigurablesTuple) -> () {
        todo!()
    }
}


pub struct HealthCheckResponse {
    feagi_running: bool,
}

//endregion

