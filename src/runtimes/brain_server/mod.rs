//! The Design Server is a runtime in which a burst engine is run that allows for dynamically
//! changing the structures via external tools (BV). This is the main backend used when designing
//! FEAGI Genomes

pub mod system_requests;
pub mod runtime_handler;
pub mod request_endpoint_grouping;