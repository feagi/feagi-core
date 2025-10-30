// ZMQ transport adapter
//
// This module provides ZMQ support by leveraging the existing api_control
// infrastructure in feagi-pns. We do NOT duplicate ZMQ code here.
//
// Architecture:
//   feagi-pns::api_control  → Handles ZMQ ROUTER/DEALER transport
//   feagi-api::transports::zmq → Provides business logic (routes to endpoints)
//
// Integration:
//   When feagi-pns::api_control receives a REST-like ZMQ message, it calls:
//   handle_api_control_request() from this module to execute the business logic.

pub mod server;

pub use server::{
    ZmqApiState,
    ZmqRequest,
    ZmqResponse,
    route_zmq_request,
    handle_api_control_request,
};
