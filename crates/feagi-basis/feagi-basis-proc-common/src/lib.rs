use proc_macro::TokenStream;

//region Independent Builders
mod bit_struct_builder;
mod feagi_id_builder;
mod typed_enum_builder;

/// Builds a bit field struct around a uint with one named bit field per flag.
///
/// # Example
///
/// ```ignore
/// use feagi_macros::bit_struct_builder;
///
/// bit_struct_builder! {
///     u8,
///     pub ExampleFlags,
///     #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
///     /// Runtime visibility and enable flags.
///     {
///         enabled,
///         visible,
///     }
/// }
///
/// let mut flags = ExampleFlags::new(0);
/// flags.set_enabled(true);
/// assert!(flags.enabled());
/// flags.toggle_visible();
/// assert!(flags.visible());
/// assert_eq!(flags.bits(), 0b0000_0011);
/// ```
#[proc_macro]
pub fn bit_struct_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bit_struct_builder::bit_struct_builder(input)
}

/// Defines a fixed-byte FeagiID tuple struct `Name([u8; N])`, implementing `FeagiID` and serde
/// (as the byte array, checked on deserialize). Options, in any order:
///
/// - `try_from = (ErrorType, check_fn)`: `check_fn: fn([u8; N]) -> Result<Name, ErrorType>`
///   becomes `try_from_id_bytes`. Without it every byte array is valid, the error is
///   `Infallible`, and a `const fn from_id_bytes` is added
/// - `random`: adds a `std`-only `new_random`, which draws until the check passes
/// - `uuid = v4` or `uuid = v7`: for 16-byte IDs holding a UUID of that version. The version is
///   the check (the error is `UuidVersionError`), `FeagiUuidID` is implemented, and the
///   version's constructors are added: `new_random` (`std`) and `new_from_random_bytes` for v4,
///   `new_now` (`std`) and `new_from_unix_millis` for v7. Not usable with `try_from` or `random`
/// - `base64` or `base64 = Base64Name`: adds the padded URL-safe base64 struct (named
///   `Base64{Name}` by default), implementing `FeagiIDBase64`, and `HasBase64` on the ID. Both
///   types convert into each other with `From`, by value or by reference
///
/// # Example
///
/// ```ignore
/// use feagi_basis::feagi_id::feagi_id;
///
/// feagi_id! {
///     /// Docs for the generated struct
///     pub NodeID: 16,
///     try_from = (NodeIDError, check_node_id),
///     random,
///     base64,
/// }
/// ```
#[proc_macro]
pub fn feagi_id(input: TokenStream) -> TokenStream {
    feagi_id_builder::feagi_id(input)
}

#[proc_macro]
pub fn enum_leveled_builder(input: TokenStream) -> TokenStream {
    typed_enum_builder::enum_leveled_builder(input)
}

//region Templates

//region Feagi Request Messaging

use feagi_macro_components::feagi_messages::request_response::feagi_request_category_group::FeagiRequestCategoryGroup;
use feagi_macro_components::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategory;


/// Defines a category of request messages. Generates, for every message, its path, request
/// specifications, response, error, message struct, and descriptor. Then for the category its
/// configurables struct, request and response enums, and the category marker implementing
/// `FeagiRequestCategory`
#[proc_macro]
pub fn define_request_message_category(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<FeagiRequestMessageCategory>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(category) => category.generate_rust_category().into(),
    }
}

//endregion

//region Feagi Request Messaging Categorization

/// Defines a group of request categories, by listing the category markers in order. Generates
/// `{Group}Requesters` and `{Group}Responders` (one field per category, generic over the channel
/// backend), `{Group}Configurables`, and the requesters' `FeagiRequestCategoryGroup` impl
#[proc_macro]
pub fn define_request_category_group(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<FeagiRequestCategoryGroup>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(group) => group.generate_rust_group().into(),
    }
}

//endregion

//endregion
