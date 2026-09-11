pub mod feagi_index_organizer_error;

#[cfg(feature = "alloc")]
pub use index_manager::IndexManager;

#[cfg(feature = "std")]
pub use bi_direction_hashmap::BiDirectionHashmap;

#[cfg(feature = "alloc")]
mod index_manager;
#[cfg(feature = "std")]
mod bi_direction_hashmap;