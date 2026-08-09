pub mod cortical_area_prev;

// `CorticalArea::new` takes this type and `CorticalArea::dimensions` returns it, so callers cannot
// use the public API without being able to name it. Re-exported here so they need not depend on
// `feagi-data` directly.
pub use feagi_data::neuron_voxels::wrapped_values::NeuronVoxelDimensionsGenomic;
