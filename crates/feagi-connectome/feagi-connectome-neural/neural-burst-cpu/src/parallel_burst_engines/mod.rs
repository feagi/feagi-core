// The engine uses rayon (so std threading), and builds on `shared_structs` and neural-basis's engine trait
//#[cfg(feature = "rayon")]
//pub mod rayon;
pub mod neural_cpu_parallel_dynamics;
pub mod synapse_cpu_parallel_dynamics;
