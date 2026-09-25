//! Parse a template DSL into token structs, validate it, and re-export that template as a
//! reusable `macro_rules!`. Later generator proc macros (final consumers) can turn those
//! structs into Rust source.
//!
//! Parser macros validate the source-file template, then emit:
//!
//! ```ignore
//! macro_rules! exported_name {
//!     ($consumer:path) => {
//!         $consumer! { /* T.expand_template() tokens */ }
//!     };
//! }
//! ```
//!
//! `exported_name!(some_consumer)` unwraps the stored template into `some_consumer`.
//! `some_consumer` then calls `TemplateRoot::parse_generator_input_macro` to rebuild `T`.
//! Proc macros do not expand their input, so the callback is required; `exported_name!()`
//! cannot be placed inside another proc-macro invocation and be unwrapped automatically.


#[cfg(feature = "request_response_macros")]
pub mod requests_responses;
pub mod template_root;