//! Rest Server stuff using picoserve. While this library is used for both std and no-std, their
//! implementations are different and in different modules, with the std version is using
//! its own thread and no-std using embassy. They do have some similar code however, namely
//! in the actual routing generation

pub mod error;
pub mod server_common;

#[cfg(feature = "open_api_endpoint")]
pub mod open_api_builder;
#[cfg(feature = "std")]
pub mod server_std;
