//! The Genome Manager Handles Genome metadata while coordinating burst engines, effectively
//! using the Genome as a check to ensure edit operations toward the NPU are valid


pub mod requests;
pub mod worker_builder;
pub mod worker_loop;
pub mod response_processing;