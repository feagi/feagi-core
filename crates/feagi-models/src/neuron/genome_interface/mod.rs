//! This module details structs / traits that are used from the outside of the NPU to pass
//! instructions or commands into the NPU and its burst engines. These structs should not 
//! "bleed" out quantizations, generics, or other parameters that would be challenging to work with
//! in the rest of FEAGI (the only exception should be for struct initialization to pass in
//! immediately into the NPU)

pub mod cortical_area_spawners;
pub mod iterators;

