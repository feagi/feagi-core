// Contract testing module
//
// These tests ensure 100% backward compatibility with the Python FastAPI implementation.
// They compare Rust API responses against snapshots from the Python API.

#[cfg(test)]
mod health_tests;

#[cfg(test)]
mod test_utils;


