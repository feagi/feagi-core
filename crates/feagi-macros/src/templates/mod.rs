//! These goal of these macros is to parse some form of template into standard (token) structs,
//! that then generator macros can then export as Rust Structs, Methods, and anything else.
//!
//! The way this works is Parser Macros process the template input in the source files, validate it
//! to the best of its ability, then exports that template as itself via a generated 
//! exported macro_rules!, such that one template can be used in many places.
//! Generator proc macros from in here can then take these macro_rules! macros and parse them to
//! produce Rust Source code


#[cfg(feature = "request_response_macros")]
pub mod requests_responses;