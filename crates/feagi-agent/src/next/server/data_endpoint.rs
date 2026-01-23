//! Data endpoint for handling sensor input and motor/visual outputs.

use feagi_io::next::implementations::zmq::{
    FEAGIZMQServerPublisherProperties, FEAGIZMQServerPullerProperties,
};
use feagi_io::next::traits_and_enums::server::server_shared::FeagiServerBindStateChange;
use feagi_io::next::traits_and_enums::server::{FeagiServer, FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller, FeagiServerPullerProperties};
use feagi_io::next::FeagiNetworkError;

/// Data endpoint that owns motor/voxel publishers and a sensor puller.
pub struct DataEndpoint {
    motor: Box<dyn FeagiServerPublisher>,
    voxel_visual: Box<dyn FeagiServerPublisher>,
    sensor: Box<dyn FeagiServerPuller>,
}

impl DataEndpoint {
    /// Create and start the data endpoint.
    pub fn new(
        motor_endpoint: String,
        voxel_visual_endpoint: String,
        sensor_endpoint: String,
    ) -> Result<Self, FeagiNetworkError> {
        let mut motor = Box::new(FEAGIZMQServerPublisherProperties::new(motor_endpoint)).build(Box::new(|change| {
            Self::handle_motor_state_change(change);
        }));
        let mut voxel_visual = Box::new(FEAGIZMQServerPublisherProperties::new(voxel_visual_endpoint)).build(Box::new(|change| {
            Self::handle_voxel_visual_state_change(change);
        }));
        let mut sensor = Box::new(FEAGIZMQServerPullerProperties::new(sensor_endpoint)).build(Box::new(|change| {
            Self::handle_sensor_state_change(change);
        }));

        motor.start()?;
        voxel_visual.start()?;
        sensor.start()?;

        Ok(Self {
            motor,
            voxel_visual,
            sensor,
        })
    }

    /// Async loop to poll all endpoints.
    #[cfg(feature = "async")]
    pub async fn run(&mut self) -> Result<(), FeagiNetworkError> {
        loop {
            self.motor.poll()?;
            self.voxel_visual.poll()?;

            let sensor_data = match self.sensor.try_poll_receive()? {
                Some(data) => Some(data.to_vec()),
                None => None,
            };

            if let Some(data) = sensor_data {
                self.handle_sensor_data(data);
            }

            tokio::task::yield_now().await;
        }
    }

    fn handle_motor_state_change(change: FeagiServerBindStateChange) {
        let _ = change;
        // TODO: handle motor publisher state changes
    }

    fn handle_voxel_visual_state_change(change: FeagiServerBindStateChange) {
        let _ = change;
        // TODO: handle voxel visual publisher state changes
    }

    fn handle_sensor_state_change(change: FeagiServerBindStateChange) {
        let _ = change;
        // TODO: handle sensor puller state changes
    }

    fn handle_sensor_data(&mut self, data: Vec<u8>) {
        let _ = data;
        // TODO: handle incoming sensor data
    }
}
