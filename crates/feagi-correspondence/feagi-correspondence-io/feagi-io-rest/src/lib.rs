//! Effectively a Router Crate that routes the correct subcrate depending on feature set

pub extern crate feagi_rest_basis;

/// Ohkami HTTP server (rest and std)
#[cfg(all(feature = "feagi-rest-server", feature = "std"))]
pub extern crate feagi_rest_server_std;


/*
/// TODO no std server
#[cfg(all(feature = "feagi-rest-server", not(feature = "std")))]
pub extern crate feagi_rest_server_no-std;
*/