//! Base SDK utilities for controller implementations.

mod controller;
mod topology;

pub use controller::Controller;
pub use topology::{CorticalTopology, TopologyCache};
