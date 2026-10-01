//! Effectively a Router Crate that routes the correct subcrate depending on feature set

// NOTE: We swap the name of the crate to a uniform one for easier upstream compatibility

// TODO we should have a REST basis crate for shared structs

// TODO maybe we should move the OpenAPI generation to its own crate due to ohkami limits and to make
// it cross platform


/// Ohkami HTTP server (rest and std)
#[cfg(all(feature = "feagi-rest-server", feature = "std"))]
pub extern crate feagi_rest_server_std as feagi_rest_server;


/*
/// TODO no std server
#[cfg(all(feature = "feagi-rest-server", not(feature = "std")))]
pub extern crate feagi_rest_server_no-std as feagi_rest_server;
*/