

// NOTE: This entire module is not exported in no-alloc / no-std contexts!

mod region_id;
mod region_type;
mod brain_region;

pub use region_id::RegionID;
pub use brain_region::BrainRegion;
pub use region_type::RegionType;

