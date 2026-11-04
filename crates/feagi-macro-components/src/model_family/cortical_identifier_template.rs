use super::cortical_type_family_template::{CorticalTypeFamilyTemplate, SuperTypeTemplate};
use proc_macro2::Ident;
use quote::{format_ident, quote};

//region Cortical Identifier

/// Generates the identifier of a cortical type family from its template, on top of the items
/// generated for the family itself. An identifier is exactly 8 bytes:
/// ```text
/// byte 0:     super type key
/// byte 1:     cortical type key, unique within its super type
/// bytes 2-7:  inner ID, whose meaning the cortical type decides
/// ```
impl CorticalTypeFamilyTemplate {
    /// The fully decoded form of the identifier
    pub(super) fn path_name(&self) -> Ident {
        format_ident!("{}Path", self.identifier_name)
    }

    pub(super) fn super_type_path_name(&self, super_type: &Ident) -> Ident {
        format_ident!("{}{}Path", super_type, self.identifier_name)
    }

    /// Generate the identifier, its error, and the nested enums it decodes to
    pub fn generate_rust_identifier(&self) -> proc_macro2::TokenStream {
        let error = self.generate_identifier_error();
        let super_type_paths = self.super_types.iter().map(|(super_type, template)| self.generate_super_type_path(super_type, template));
        let path = self.generate_path();
        let identifier = self.generate_identifier();
        quote! {
            #error
            #( #super_type_paths )*
            #path
            #identifier
        }
    }

    fn generate_identifier_error(&self) -> proc_macro2::TokenStream {
        let identifier = &self.identifier_name;
        let error = &self.identifier_error_name;
        let family_kind = self.family_kind_name();
        let doc = format!("Why bytes are not a valid `{identifier}`, naming the first layer that rejected them");
        quote! {
            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub enum #error {
                /// No super type has this key
                SuperType(u8),
                /// The super type has no cortical type with this key
                CorticalType { super_type: u8, cortical_type: u8 },
                /// The cortical type rejected its inner ID
                Inner(#family_kind),
            }

            impl ::core::fmt::Display for #error {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        Self::SuperType(key) => write!(f, "No super type has the key {}", key),
                        Self::CorticalType { super_type, cortical_type } => write!(f, "The super type of key {} has no cortical type of key {}", super_type, cortical_type),
                        Self::Inner(kind) => write!(f, "The inner ID is not valid for {}", kind),
                    }
                }
            }

            impl ::core::error::Error for #error {}
        }
    }

    /// `{SuperType}{Identifier}Path`, with a variant per cortical type holding its inner ID
    fn generate_super_type_path(&self, super_type: &Ident, template: &SuperTypeTemplate) -> proc_macro2::TokenStream {
        let identifier = &self.identifier_name;
        let error = &self.identifier_error_name;
        let family_kind = self.family_kind_name();
        let kind = Self::super_type_kind_name(super_type);
        let path = self.super_type_path_name(super_type);
        let base_trait = self.base_trait_path();
        let super_type_key = template.key;

        let names: Vec<&Ident> = template.cortical_types.keys().collect();
        let attributes = template.cortical_types.values().map(|cortical_type| &cortical_type.attributes);
        let keys: Vec<u8> = template.cortical_types.values().map(|cortical_type| cortical_type.key).collect();
        let inners: Vec<proc_macro2::TokenStream> = names
            .iter()
            .map(|name| {
                let cortical_type = self.cortical_type_path(super_type, name);
                quote!(<#cortical_type as #base_trait>::Inner)
            })
            .collect();
        let doc = format!("A decoded `{identifier}` of the `{super_type}` super type");

        quote! {
            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
            pub enum #path {
                #(
                    #( #attributes )*
                    #names(#inners),
                )*
            }

            impl #path {
                /// The cortical type this identifies
                pub const fn kind(&self) -> #kind {
                    match self {
                        #( Self::#names(_) => #kind::#names, )*
                    }
                }

                /// (cortical type key, inner bytes)
                fn parts(self) -> (u8, [u8; 6]) {
                    match self {
                        #( Self::#names(inner) => (#keys, ::core::convert::Into::<[u8; 6]>::into(inner)), )*
                    }
                }

                fn from_parts(cortical_type: u8, inner: [u8; 6]) -> ::core::result::Result<Self, #error> {
                    match cortical_type {
                        #(
                            #keys => <#inners as ::core::convert::TryFrom<[u8; 6]>>::try_from(inner)
                                .map(Self::#names)
                                .map_err(|_| #error::Inner(#family_kind::#super_type(#kind::#names))),
                        )*
                        cortical_type => ::core::result::Result::Err(#error::CorticalType { super_type: #super_type_key, cortical_type }),
                    }
                }
            }
        }
    }

    /// `{Identifier}Path`, with a variant per super type holding its path
    fn generate_path(&self) -> proc_macro2::TokenStream {
        let identifier = &self.identifier_name;
        let error = &self.identifier_error_name;
        let family_kind = self.family_kind_name();
        let path = self.path_name();

        let names: Vec<&Ident> = self.super_types.keys().collect();
        let attributes = self.super_types.values().map(|super_type| &super_type.attributes);
        let keys: Vec<u8> = self.super_types.values().map(|super_type| super_type.key).collect();
        let super_type_paths: Vec<Ident> = names.iter().map(|name| self.super_type_path_name(name)).collect();
        let doc = format!("The fully decoded form of a `{identifier}`, which is what one is matched as, never what is stored");

        quote! {
            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
            pub enum #path {
                #(
                    #( #attributes )*
                    #names(#super_type_paths),
                )*
            }

            impl #path {
                /// The cortical type this identifies
                pub const fn kind(&self) -> #family_kind {
                    match self {
                        #( Self::#names(inner) => #family_kind::#names(inner.kind()), )*
                    }
                }

                /// The bytes of the identifier
                pub fn to_bytes(self) -> [u8; 8] {
                    let (super_type, (cortical_type, inner)) = match self {
                        #( Self::#names(inner) => (#keys, inner.parts()), )*
                    };
                    let [a, b, c, d, e, f] = inner;
                    [super_type, cortical_type, a, b, c, d, e, f]
                }

                /// Unrolls the bytes through each layer of enums in order
                pub fn try_from_bytes(bytes: [u8; 8]) -> ::core::result::Result<Self, #error> {
                    let [super_type, cortical_type, a, b, c, d, e, f] = bytes;
                    let inner = [a, b, c, d, e, f];
                    match super_type {
                        #( #keys => #super_type_paths::from_parts(cortical_type, inner).map(Self::#names), )*
                        super_type => ::core::result::Result::Err(#error::SuperType(super_type)),
                    }
                }
            }
        }
    }

    /// The stored identifier, whose private fields only ever hold valid bytes
    fn generate_identifier(&self) -> proc_macro2::TokenStream {
        let identifier = &self.identifier_name;
        let error = &self.identifier_error_name;
        let family_name = &self.family_name;
        let family_kind = self.family_kind_name();
        let member_trait = self.member_trait_name();
        let base_trait = self.base_trait_path();
        let path = self.path_name();

        let leaves = self.cortical_types();
        let super_type_keys = leaves.iter().map(|leaf| self.super_types[leaf.super_type].key);
        let cortical_type_keys = leaves.iter().map(|leaf| leaf.template.key);
        let kinds = leaves.iter().map(|leaf| self.kind_value(leaf.super_type, leaf.name));
        let doc = format!(
            "Identifies a cortical area of the `{family_name}` family, in exactly 8 bytes, which is what gets stored, hashed, and sent. Byte 0 is the super type key, byte 1 the cortical type key, and bytes 2 to 7 the inner ID. Only valid bytes can be held, and as keys are never 0, `Option<{identifier}>` is 8 bytes as well. Serialized as its 8 bytes, whichever the format"
        );

        quote! {
            #[doc = #doc]
            #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
            pub struct #identifier {
                super_type: ::core::num::NonZeroU8,
                rest: [u8; 7],
            }

            const _: () = ::core::assert!(::core::mem::size_of::<#identifier>() == 8);
            const _: () = ::core::assert!(::core::mem::size_of::<::core::option::Option<#identifier>>() == 8);

            impl #identifier {
                /// Identifies an area of cortical type `T`
                ///
                /// # Panics
                ///
                /// If `T::Inner` does not return the exact same bytes it was made from
                pub fn new<T: #member_trait>(inner: <T as #base_trait>::Inner) -> Self {
                    let [a, b, c, d, e, f] = ::core::convert::Into::<[u8; 6]>::into(inner);
                    Self::from_valid_bytes([<T as #member_trait>::KIND.super_type_key(), <T as #member_trait>::KIND.key(), a, b, c, d, e, f])
                }

                /// # Panics
                ///
                /// If an inner ID does not return the exact same bytes it was made from
                pub fn from_path(path: #path) -> Self {
                    Self::from_valid_bytes(path.to_bytes())
                }

                fn from_valid_bytes(bytes: [u8; 8]) -> Self {
                    match Self::try_from_bytes(bytes) {
                        ::core::result::Result::Ok(identifier) => identifier,
                        ::core::result::Result::Err(_) => ::core::unreachable!("an inner ID did not round trip through its bytes"),
                    }
                }

                pub fn try_from_bytes(bytes: [u8; 8]) -> ::core::result::Result<Self, #error> {
                    #path::try_from_bytes(bytes)?;
                    let [super_type, a, b, c, d, e, f, g] = bytes;
                    match ::core::num::NonZeroU8::new(super_type) {
                        ::core::option::Option::Some(super_type) => ::core::result::Result::Ok(Self { super_type, rest: [a, b, c, d, e, f, g] }),
                        ::core::option::Option::None => ::core::result::Result::Err(#error::SuperType(0)),
                    }
                }

                pub const fn to_bytes(self) -> [u8; 8] {
                    let [a, b, c, d, e, f, g] = self.rest;
                    [self.super_type.get(), a, b, c, d, e, f, g]
                }

                /// The nested enums, to match on
                pub fn path(self) -> #path {
                    match #path::try_from_bytes(self.to_bytes()) {
                        ::core::result::Result::Ok(path) => path,
                        ::core::result::Result::Err(_) => ::core::unreachable!("only valid bytes are held"),
                    }
                }

                /// The cortical type this identifies, without decoding the inner ID
                pub const fn kind(self) -> #family_kind {
                    match (self.super_type.get(), self.rest[0]) {
                        #( (#super_type_keys, #cortical_type_keys) => #kinds, )*
                        _ => ::core::unreachable!(),
                    }
                }

                /// The inner ID as bytes, without decoding it
                pub const fn inner_bytes(self) -> [u8; 6] {
                    let [_, a, b, c, d, e, f] = self.rest;
                    [a, b, c, d, e, f]
                }
            }

            impl ::serde::Serialize for #identifier {
                fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> ::core::result::Result<S::Ok, S::Error> {
                    ::serde::Serialize::serialize(&self.to_bytes(), serializer)
                }
            }

            impl<'de> ::serde::Deserialize<'de> for #identifier {
                fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> ::core::result::Result<Self, D::Error> {
                    let bytes = <[u8; 8] as ::serde::Deserialize>::deserialize(deserializer)?;
                    Self::try_from_bytes(bytes).map_err(<D::Error as ::serde::de::Error>::custom)
                }
            }
        }
    }
}

//endregion
