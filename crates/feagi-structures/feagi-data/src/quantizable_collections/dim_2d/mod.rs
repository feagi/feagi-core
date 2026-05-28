mod vector_dense;
mod hashmap_sparse;
pub mod spatial_shared_traits;

pub use vector_dense::QuantizableSpatialCollection2DVectorDense;
pub use hashmap_sparse::QuantizableSpatialCollection2DHashmapSparse;
// TODO sparse vector implementation with pagination?
