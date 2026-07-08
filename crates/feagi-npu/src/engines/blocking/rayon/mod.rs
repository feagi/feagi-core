// TODO obviously we are not keeping rayon here, this is just for now

mod engine;
pub mod kernels;
pub mod data;

pub use engine::RayonBurstEngine;