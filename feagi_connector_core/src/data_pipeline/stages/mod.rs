/// Module of all StreamCacheProcessors, which handle the direct caching, filtering, and transforming of data 
/// going in and out of FEAGI

mod identities;
mod rolling_windows;
mod ranges;
mod image_transformer;
mod image_segmentor;
mod image_quick_diff;

pub use identities::*;
pub use rolling_windows::*;
pub use ranges::*;
pub use image_transformer::*;
pub use image_quick_diff::*;
pub use image_segmentor::*;