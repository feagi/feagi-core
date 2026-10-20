pub mod common;
pub mod feagi_messages;
pub mod template_parsing;
pub mod templates;

/// Template maps keep their entries in template order, so generated fields, variants, and type
/// lists come out in the same order on every build
pub use indexmap::IndexMap;
