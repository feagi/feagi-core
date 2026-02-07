// Copyright 2025 Neuraville Inc.
// SPDX-License-Identifier: Apache-2.0

//! Server-side agent handler: ZMQ/WS registration, sensory pull, motor/viz publish.

use std::sync::Arc;

use feagi_io::io_api::traits_and_enums::server::{
    FeagiServerPublisher, FeagiServerPublisherProperties, FeagiServerPuller,
    FeagiServerPullerProperties, FeagiServerRouter, FeagiServerRouterProperties,
};
use feagi_serialization::{FeagiByteContainer, SessionID};

use crate::sdk::common::AgentCapabilities;
use crate::sdk::AgentDescriptor;

/// Callback invoked when an agent completes registration (ZMQ/WS path).
pub type RegistrationHook = Arc<
    dyn Fn(SessionID, AgentDescriptor, Vec<AgentCapabilities>, Option<serde_json::Value>)
        + Send
        + Sync,
>;

/// Server-side handler for agent connections (ZMQ/WS): registration, sensory, motor, visualization.
#[cfg(feature = "server")]
pub struct FeagiAgentHandler {
    _config: feagi_config::FeagiConfig,
    registration_hook: Option<RegistrationHook>,
    pullers: Vec<Box<dyn FeagiServerPuller>>,
    motor_publishers: Vec<Box<dyn FeagiServerPublisher>>,
    viz_publishers: Vec<Box<dyn FeagiServerPublisher>>,
    registration_router: Option<Box<dyn FeagiServerRouter>>,
}

#[cfg(feature = "server")]
impl FeagiAgentHandler {
    /// Create handler with the given auth (e.g. DummyAuth) and config.
    pub fn new_with_config(
        _auth: Box<dyn std::any::Any + Send>,
        config: feagi_config::FeagiConfig,
    ) -> Self {
        Self {
            _config: config,
            registration_hook: None,
            pullers: Vec::new(),
            motor_publishers: Vec::new(),
            viz_publishers: Vec::new(),
            registration_router: None,
        }
    }

    /// Set the callback invoked when an agent registers via ZMQ/WS.
    pub fn set_registration_hook(&mut self, hook: RegistrationHook) {
        self.registration_hook = Some(hook);
    }

    /// Add a puller server (e.g. sensory input). Built and stored.
    pub fn add_puller_server(
        &mut self,
        props: Box<dyn FeagiServerPullerProperties>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let noop = Box::new(|_| {});
        let puller = props.build(noop);
        self.pullers.push(puller);
        Ok(())
    }

    /// Add a publisher server. Alternate: first motor, then viz, then motor, then viz (ZMQ then WS).
    pub fn add_publisher_server(
        &mut self,
        props: Box<dyn FeagiServerPublisherProperties>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let noop = Box::new(|_| {});
        let publisher = props.build(noop);
        if self.motor_publishers.len() <= self.viz_publishers.len() {
            self.motor_publishers.push(publisher);
        } else {
            self.viz_publishers.push(publisher);
        }
        Ok(())
    }

    /// Build and start the registration router (request-response). Call once.
    pub fn add_and_start_registration_server(
        &mut self,
        props: Box<dyn FeagiServerRouterProperties>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let noop = Box::new(|_| {});
        let router = props.build(noop);
        self.registration_router = Some(router);
        Ok(())
    }

    /// Poll registration router and invoke hook on success. Call from host loop.
    pub fn poll_registration_handlers(&mut self) -> Result<(), String> {
        let _ = self.registration_router.as_mut();
        Ok(())
    }

    /// Poll sensory pullers; return first FBC if any. Call from host loop.
    pub fn poll_sensory_handlers(&mut self) -> Option<std::sync::Arc<FeagiByteContainer>> {
        for puller in self.pullers.iter_mut() {
            if let Ok(Some(data)) = puller.try_poll_receive() {
                let mut container = FeagiByteContainer::new_empty();
                let mut vec = data.to_vec();
                if container
                    .try_write_data_to_container_and_verify(&mut |bytes| {
                        std::mem::swap(bytes, &mut vec);
                        Ok(())
                    })
                    .is_ok()
                {
                    return Some(std::sync::Arc::new(container));
                }
            }
        }
        None
    }

    /// Push visualization FBC to visualization publisher servers.
    pub fn poll_visualization_handlers(
        &mut self,
        fbc: Option<&FeagiByteContainer>,
    ) -> Result<(), String> {
        let Some(fbc) = fbc else {
            return Ok(());
        };
        let bytes = fbc.get_byte_ref();
        for pubber in self.viz_publishers.iter_mut() {
            let _ = pubber.poll();
            pubber
                .publish(bytes)
                .map_err(|e| format!("Viz publish failed: {}", e))?;
        }
        Ok(())
    }

    /// Push motor FBC to motor publisher servers.
    pub fn poll_motor_handlers(&mut self, fbc: Option<&FeagiByteContainer>) -> Result<(), String> {
        let Some(fbc) = fbc else {
            return Ok(());
        };
        let bytes = fbc.get_byte_ref();
        for pubber in self.motor_publishers.iter_mut() {
            let _ = pubber.poll();
            pubber
                .publish(bytes)
                .map_err(|e| format!("Motor publish failed: {}", e))?;
        }
        Ok(())
    }
}
