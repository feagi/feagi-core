use feagi_io::FeagiNetworkError;
use feagi_io::implementations::zmq::{FEAGIZMQClientPusher, FEAGIZMQClientRequester, FEAGIZMQClientSubscriber};
use feagi_io::traits_and_enums::client::{FeagiClientPusher, FeagiClientSubscriber};
use crate::clients::registration_agent::RegistrationAgent;
use crate::FeagiAgentClientError;
use crate::registration::{AgentCapabilities, AgentDescriptor, AuthToken, ConnectionProtocol, RegistrationRequest};

pub struct ConnectorAgent {
    sensor_pusher: Box<dyn FeagiClientPusher>,
    motor_subscriber: Box<dyn FeagiClientSubscriber>
}

impl ConnectorAgent {
    pub async fn new(feagi_registration_endpoint: String, agent_descriptor: AgentDescriptor, auth_token: AuthToken) -> Result<Self, FeagiAgentClientError> {

        // TODO hardcoded for now
        let mut registration_agent = RegistrationAgent::new(
            Box::new(FEAGIZMQClientRequester::new(
                feagi_registration_endpoint,
                Box::new(|_| {}),
            ).map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?)
        );

        let registration_request = RegistrationRequest::new(
            agent_descriptor,
            auth_token,
            vec![
                AgentCapabilities::ReceiveMotorData,
                AgentCapabilities::SendSensorData
            ],
            ConnectionProtocol::ZMQ
        );

        let (session_id, endpoints) = registration_agent.try_register(registration_request).await?;

        let sensor_endpoint = endpoints.get(&AgentCapabilities::SendSensorData).unwrap();
        let motor_endpoint = endpoints.get(&AgentCapabilities::ReceiveMotorData).unwrap();

        let mut sensor_pusher: Box<dyn FeagiClientPusher> = Box::new(
            FEAGIZMQClientPusher::new(
                sensor_endpoint.clone(),
                Box::new(|_| {}),
            ).map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?
        );

        let mut motor_subscriber: Box<dyn FeagiClientSubscriber> = Box::new(
            FEAGIZMQClientSubscriber::new(
                motor_endpoint.clone(),
                Box::new(|_| {}),
            ).map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?
        );

        // TODO whats going on with the multiple host name definitions?

        sensor_pusher.connect(&sensor_endpoint).await
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;
        motor_subscriber.connect(&motor_endpoint).await
            .map_err(|e| FeagiAgentClientError::ConnectionFailed(e.to_string()))?;

        Ok(ConnectorAgent {
            sensor_pusher,
            motor_subscriber
        })
    }

    pub async fn push_sensor_data(&mut self, data: &[u8]) -> Result<(), FeagiNetworkError> {
        self.sensor_pusher.push_data(data).await
    }

    pub async fn poll_motor_data(&mut self) -> Result<Vec<u8>, FeagiNetworkError> {
        self.motor_subscriber.get_subscribed_data().await
    }
}