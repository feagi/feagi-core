//! Generates a fixed-byte FeagiID tuple struct, optionally with random construction, a UUID
//! version, and a base64 form.

use feagi_macro_components::common::parse_optional_comma;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, parse_macro_input, Attribute, Expr, Ident, LitStr, Path, Token, Type, Visibility};

pub fn feagi_id(input: TokenStream) -> TokenStream {
    let spec = parse_macro_input!(input as FeagiIDSpec);
    TokenStream::from(spec.expand())
}

/// A custom `try_from_id_bytes`: `fn([u8; N]) -> Result<Name, ErrorType>`
struct TryFromOverride {
    error_type: Type,
    check: Path,
}

/// The UUID version a 16-byte ID holds
#[derive(Clone, Copy)]
enum UuidVersion {
    V4,
    V7,
}

struct FeagiIDSpec {
    attrs: Vec<Attribute>,
    vis: Visibility,
    name: Ident,
    byte_count: Expr,
    try_from: Option<TryFromOverride>,
    random: bool,
    uuid: Option<UuidVersion>,
    /// The name of the base64 struct, if one is wanted
    base64: Option<Ident>,
}

impl Parse for FeagiIDSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis: Visibility = input.parse()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let byte_count: Expr = input.parse()?;
        parse_optional_comma(input)?;

        let mut try_from = None;
        let mut random = false;
        let mut uuid = None;
        let mut base64 = None;

        while !input.is_empty() {
            let option: Ident = input.parse()?;
            match option.to_string().as_str() {
                "try_from" => {
                    reject_repeat(try_from.is_some(), &option)?;
                    input.parse::<Token![=]>()?;
                    let content;
                    parenthesized!(content in input);
                    let error_type: Type = content.parse()?;
                    content.parse::<Token![,]>()?;
                    let check: Path = content.parse()?;
                    parse_optional_comma(&content)?;
                    try_from = Some(TryFromOverride { error_type, check });
                    if uuid.is_some() {
                        return Err(syn::Error::new(option.span(), UUID_WITH_TRY_FROM));
                    }
                }
                "random" => {
                    reject_repeat(random, &option)?;
                    random = true;
                    if uuid.is_some() {
                        return Err(syn::Error::new(option.span(), UUID_WITH_RANDOM));
                    }
                }
                "uuid" => {
                    reject_repeat(uuid.is_some(), &option)?;
                    input.parse::<Token![=]>()?;
                    let version: Ident = input.parse()?;
                    uuid = Some(match version.to_string().as_str() {
                        "v4" => UuidVersion::V4,
                        "v7" => UuidVersion::V7,
                        _ => return Err(syn::Error::new(version.span(), "feagi_id: `uuid` must be `v4` or `v7`")),
                    });
                    if try_from.is_some() {
                        return Err(syn::Error::new(option.span(), UUID_WITH_TRY_FROM));
                    }
                    if random {
                        return Err(syn::Error::new(option.span(), UUID_WITH_RANDOM));
                    }
                }
                "base64" => {
                    reject_repeat(base64.is_some(), &option)?;
                    base64 = Some(if input.peek(Token![=]) {
                        input.parse::<Token![=]>()?;
                        input.parse::<Ident>()?
                    } else {
                        format_ident!("Base64{}", name)
                    });
                }
                _ => {
                    return Err(syn::Error::new(
                        option.span(),
                        "feagi_id: expected one of `try_from = (ErrorType, check_fn)`, `random`, `uuid = v4`, `uuid = v7`, `base64`, or `base64 = Name`",
                    ));
                }
            }
            parse_optional_comma(input)?;
        }

        Ok(Self {
            attrs,
            vis,
            name,
            byte_count,
            try_from,
            random,
            uuid,
            base64,
        })
    }
}

const UUID_WITH_TRY_FROM: &str = "feagi_id: `uuid` is the check of the ID, so it cannot be used with `try_from`";
const UUID_WITH_RANDOM: &str = "feagi_id: `uuid` adds its own constructors, so it cannot be used with `random`";

fn reject_repeat(already_set: bool, option: &Ident) -> syn::Result<()> {
    if already_set {
        return Err(syn::Error::new(option.span(), format!("feagi_id: `{}` was given more than once", option)));
    }
    Ok(())
}

impl FeagiIDSpec {
    fn expand(self) -> proc_macro2::TokenStream {
        let Self {
            attrs,
            vis,
            name,
            byte_count,
            try_from,
            random,
            uuid,
            base64,
        } = self;

        let basis = quote!(::feagi_basis);
        let serde = quote!(#basis::__private::serde);
        let feagi_id_trait = quote!(#basis::feagi_id::feagi_id::FeagiID<{ #byte_count }>);

        let (error_type, try_from_body, const_constructor) = match (&try_from, uuid) {
            (Some(TryFromOverride { error_type, check }), _) => (quote!(#error_type), quote!(#check(bytes)), quote!()),
            (None, Some(version)) => {
                let version = uuid_version_tokens(version);
                (
                    quote!(#basis::feagi_id::uuid::UuidVersionError),
                    quote! {
                        match #basis::feagi_id::uuid::check_uuid_version(&bytes, #version) {
                            ::core::result::Result::Ok(()) => ::core::result::Result::Ok(Self(bytes)),
                            ::core::result::Result::Err(error) => ::core::result::Result::Err(error),
                        }
                    },
                    quote!(),
                )
            }
            // With no check every byte array is valid, so a const constructor cannot break anything
            (None, None) => (
                quote!(::core::convert::Infallible),
                quote!(::core::result::Result::Ok(Self(bytes))),
                quote! {
                    impl #name {
                        /// Wraps `bytes` as this ID. Every byte array is valid for this ID, so
                        /// this is the same as `try_from_id_bytes` but usable in const context
                        #vis const fn from_id_bytes(bytes: [u8; #byte_count]) -> Self {
                            Self(bytes)
                        }
                    }
                },
            ),
        };

        let random_constructor = if random {
            quote! {
                impl #name {
                    #basis::__feagi_if_std! {{
                        /// Builds an ID from random bytes, drawing again until the bytes pass
                        /// `try_from_id_bytes`
                        #[allow(irrefutable_let_patterns)]
                        pub fn new_random() -> Self {
                            loop {
                                let mut bytes = [0u8; #byte_count];
                                #basis::__private::rand::fill(&mut bytes[..]);
                                if let ::core::result::Result::Ok(id) = <Self as #feagi_id_trait>::try_from_id_bytes(bytes) {
                                    return id;
                                }
                            }
                        }
                    } else {}}
                }
            }
        } else {
            quote!()
        };

        let uuid_items = uuid.map(|version| expand_uuid(&name, &byte_count, version));

        let base64_items = base64.map(|base64_name| expand_base64(&vis, &name, &byte_count, &base64_name));

        quote! {
            #(#attrs)*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #vis struct #name([u8; #byte_count]);

            #[allow(unused_braces)]
            impl #feagi_id_trait for #name {
                type DeserializationError = #error_type;

                fn get_id_bytes(&self) -> &[u8; #byte_count] {
                    &self.0
                }

                fn into_id_bytes(self) -> [u8; #byte_count] {
                    self.0
                }

                fn try_from_id_bytes(bytes: [u8; #byte_count]) -> ::core::result::Result<Self, Self::DeserializationError> {
                    #try_from_body
                }
            }

            // Serialized as the byte array. Deserializing runs the check
            impl #serde::Serialize for #name {
                fn serialize<S: #serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
                    #serde::Serialize::serialize(&self.0, serializer)
                }
            }

            #[allow(unused_braces)]
            impl<'de> #serde::Deserialize<'de> for #name {
                fn deserialize<D: #serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
                    let bytes = <[u8; #byte_count] as #serde::Deserialize>::deserialize(deserializer)?;
                    <Self as #feagi_id_trait>::try_from_id_bytes(bytes).map_err(<D::Error as #serde::de::Error>::custom)
                }
            }

            #const_constructor
            #random_constructor
            #uuid_items
            #base64_items
        }
    }
}

fn uuid_version_tokens(version: UuidVersion) -> proc_macro2::TokenStream {
    match version {
        UuidVersion::V4 => quote!(::feagi_basis::__private::uuid::Version::Random),
        UuidVersion::V7 => quote!(::feagi_basis::__private::uuid::Version::SortRand),
    }
}

fn expand_uuid(name: &Ident, byte_count: &Expr, version: UuidVersion) -> proc_macro2::TokenStream {
    let basis = quote!(::feagi_basis);
    let uuid = quote!(#basis::__private::uuid);
    let version_tokens = uuid_version_tokens(version);

    // The constructors build a UUID of the right version, so they always pass the check
    let constructors = match version {
        UuidVersion::V4 => quote! {
            #basis::__feagi_if_std! {{
                /// Builds an ID from a new random v4 UUID
                pub fn new_random() -> Self {
                    let mut bytes = [0u8; 16];
                    #basis::__private::rand::fill(&mut bytes[..]);
                    Self::new_from_random_bytes(bytes)
                }
            } else {}}

            /// Builds a v4 UUID from `random_bytes`, overwriting its version and variant bits
            pub const fn new_from_random_bytes(random_bytes: [u8; 16]) -> Self {
                Self(#uuid::Builder::from_random_bytes(random_bytes).into_uuid().into_bytes())
            }
        },
        UuidVersion::V7 => quote! {
            #basis::__feagi_if_std! {{
                /// Builds an ID from a new time-ordered v7 UUID
                pub fn new_now() -> Self {
                    Self(#uuid::Uuid::now_v7().into_bytes())
                }
            } else {}}

            /// Builds a v7 UUID from a unix timestamp in milliseconds and 10 counter / random bytes
            pub const fn new_from_unix_millis(unix_millis: u64, counter_random_bytes: &[u8; 10]) -> Self {
                Self(#uuid::Builder::from_unix_timestamp_millis(unix_millis, counter_random_bytes).into_uuid().into_bytes())
            }
        },
    };

    quote! {
        const _: () = ::core::assert!(#byte_count == 16, "feagi_id: a `uuid` ID must be 16 bytes");

        impl #basis::feagi_id::uuid::FeagiUuidID for #name {
            const VERSION: #uuid::Version = #version_tokens;
        }

        impl #name {
            #constructors
        }
    }
}

fn expand_base64(vis: &Visibility, name: &Ident, byte_count: &Expr, base64_name: &Ident) -> proc_macro2::TokenStream {
    let basis = quote!(::feagi_basis);
    let serde = quote!(#basis::__private::serde);
    let base64_byte_count = quote!({ usize::div_ceil(#byte_count, 3) * 4 });
    let feagi_id_trait = quote!(#basis::feagi_id::feagi_id::FeagiID);
    let base64_trait = quote!(#basis::feagi_id::base_64::FeagiIDBase64<{ #byte_count }, #base64_byte_count>);
    let error_type = quote!(#basis::feagi_id::base_64::FeagiIDBase64Error<<#name as #feagi_id_trait<{ #byte_count }>>::DeserializationError>);
    let doc = LitStr::new(&format!("The padded URL-safe base64 form of [`{}`]", name), Span::call_site());

    quote! {
        #[doc = #doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #vis struct #base64_name([u8; #base64_byte_count]);

        #[allow(unused_braces)]
        impl #feagi_id_trait<#base64_byte_count> for #base64_name {
            type DeserializationError = #error_type;

            fn get_id_bytes(&self) -> &[u8; #base64_byte_count] {
                &self.0
            }

            fn into_id_bytes(self) -> [u8; #base64_byte_count] {
                self.0
            }

            fn try_from_id_bytes(bytes: [u8; #base64_byte_count]) -> ::core::result::Result<Self, Self::DeserializationError> {
                <Self as #base64_trait>::decode_original(&bytes)?;
                ::core::result::Result::Ok(Self(bytes))
            }
        }

        // SAFETY: the only constructors are from_original, which encodes a valid original, and
        // try_from_id_bytes, which checks the text decodes canonically into a valid original
        #[allow(unused_braces)]
        unsafe impl #base64_trait for #base64_name {
            type Original = #name;

            fn from_original(original: &Self::Original) -> Self {
                Self(<Self as #base64_trait>::encode_original(original))
            }
        }

        #[allow(unused_braces)]
        impl #basis::feagi_id::feagi_id::HasBase64<{ #byte_count }, #base64_byte_count> for #name {
            type Base64Type = #base64_name;
        }

        impl ::core::convert::From<#name> for #base64_name {
            fn from(original: #name) -> Self {
                <Self as #base64_trait>::from_original(&original)
            }
        }

        impl ::core::convert::From<&#name> for #base64_name {
            fn from(original: &#name) -> Self {
                <Self as #base64_trait>::from_original(original)
            }
        }

        impl ::core::convert::From<#base64_name> for #name {
            fn from(base64: #base64_name) -> Self {
                <#base64_name as #base64_trait>::to_original(&base64)
            }
        }

        impl ::core::convert::From<&#base64_name> for #name {
            fn from(base64: &#base64_name) -> Self {
                <#base64_name as #base64_trait>::to_original(base64)
            }
        }

        impl ::core::str::FromStr for #base64_name {
            type Err = #error_type;

            fn from_str(text: &str) -> ::core::result::Result<Self, Self::Err> {
                <Self as #base64_trait>::try_from_str(text)
            }
        }

        impl<'a> ::core::convert::TryFrom<&'a str> for #base64_name {
            type Error = #error_type;

            fn try_from(text: &'a str) -> ::core::result::Result<Self, Self::Error> {
                <Self as #base64_trait>::try_from_str(text)
            }
        }

        impl<const CAP: usize> ::core::convert::TryFrom<&#basis::__private::heapless::String<CAP>> for #base64_name {
            type Error = #error_type;

            fn try_from(text: &#basis::__private::heapless::String<CAP>) -> ::core::result::Result<Self, Self::Error> {
                <Self as #base64_trait>::try_from_heapless(text)
            }
        }

        impl ::core::convert::AsRef<str> for #base64_name {
            fn as_ref(&self) -> &str {
                <Self as #base64_trait>::as_str(self)
            }
        }

        impl ::core::fmt::Display for #base64_name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(<Self as #base64_trait>::as_str(self))
            }
        }

        impl #basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameterElement for #base64_name {
            const KIND: #basis::feagi_message::feagi_message_description::FeagiFieldKind =
                #basis::feagi_message::feagi_message_description::FeagiFieldKind::String;

            fn write_path_segment<W: ::core::fmt::Write>(&self, out: &mut W) -> ::core::fmt::Result {
                out.write_str(<Self as #base64_trait>::as_str(self))
            }
        }

        impl #basis::feagi_message::feagi_message_path::configurables::FeagiMessagePathStaticElement for #base64_name {
            fn to_str_slice(&self) -> &str {
                <Self as #base64_trait>::as_str(self)
            }
        }

        impl #serde::Serialize for #base64_name {
            fn serialize<S: #serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
                serializer.serialize_str(<Self as #base64_trait>::as_str(self))
            }
        }

        impl<'de> #serde::Deserialize<'de> for #base64_name {
            fn deserialize<D: #serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
                #basis::feagi_id::base_64::deserialize_base_64::<D, Self, { #byte_count }, #base64_byte_count>(deserializer)
            }
        }
    }
}
