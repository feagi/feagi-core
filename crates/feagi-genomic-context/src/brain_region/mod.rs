// NOTE: This entire module is not exported in no-alloc / no-std contexts!

mod brain_region;
mod region_id;
mod region_type;

pub use brain_region::BrainRegion;
pub use region_id::RegionID;
pub use region_type::RegionType;
