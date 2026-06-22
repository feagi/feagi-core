mod vector_dense;
mod hashmap_sparse;
pub mod spatial_shared_traits;

pub use vector_dense::QuantizableSpatialCollection4DVectorDense;
pub use hashmap_sparse::QuantizableSpatialCollection4DHashmapSparse;
// TODO sparse vector implementation with pagination?
