// The engine uses std threading, and builds on `shared_structs` and neural-basis's engine trait
#[cfg(feature = "std")]
pub mod rayon;
pub mod neural_cpu_parallel_dynamics;
pub mod neural_cpu_neurogenesis_dispatch;
pub mod synapse_cpu_parallel_dynamics;
