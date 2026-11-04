use super::cortical_type_family_template::{CorticalTypeFamilyTemplate, SuperTypeTemplate};
use heck::ToSnakeCase;
use proc_macro2::Ident;
use quote::{format_ident, quote};

//region Cortical Area Map

/// Generates the map of cortical areas of a cortical type family from its template, on top of the
/// items generated for the family and its identifier. Areas are held by super type, then cortical
/// type, then the inner ID of the area, so what is held for each cortical type may differ, and can
/// never disagree with the identifier it is found by
impl CorticalTypeFamilyTemplate {
    fn map_value_trait_name(&self) -> Ident {
        format_ident!("{}MapValue", self.family_name)
    }

    fn map_member_trait_name(&self) -> Ident {
        format_ident!("{}MapMember", self.family_name)
    }

    fn map_name(&self) -> Ident {
        format_ident!("{}Map", self.family_name)
    }

    fn ref_name(&self) -> Ident {
        format_ident!("{}Ref", self.family_name)
    }

    fn ref_mut_name(&self) -> Ident {
        format_ident!("{}RefMut", self.family_name)
    }

    fn super_type_map_name(super_type: &Ident) -> Ident {
        format_ident!("{}SuperTypeMap", super_type)
    }

    fn super_type_ref_name(super_type: &Ident) -> Ident {
        format_ident!("{}SuperTypeRef", super_type)
    }

    fn super_type_ref_mut_name(super_type: &Ident) -> Ident {
        format_ident!("{}SuperTypeRefMut", super_type)
    }

    /// The field of a super type or cortical type, which is its snake case
    fn map_field_name(name: &Ident) -> Ident {
        format_ident!("{}", name.to_string().to_snake_case())
    }

    /// Generate the map of the family, with the enums of what it finds
    pub fn generate_rust_area_map(&self) -> proc_macro2::TokenStream {
        let value_trait = self.generate_map_value_trait();
        let super_types = self.super_types.iter().map(|(super_type, template)| self.generate_super_type_map(super_type, template));
        let family_refs = self.generate_family_refs();
        let map = self.generate_map();
        let member = self.generate_map_member_trait();
        quote! {
            #value_trait
            #( #super_types )*
            #family_refs
            #map
            #member
        }
    }

    fn generate_map_value_trait(&self) -> proc_macro2::TokenStream {
        let value_trait = self.map_value_trait_name();
        let member_trait = self.member_trait_name();
        let family_name = &self.family_name;
        let map = self.map_name();
        let doc = format!(
            "What a [`{map}`] holds per area of each cortical type of the `{family_name}` family. Meant to be implemented on a zero sized struct per kind of map"
        );
        quote! {
            #[doc = #doc]
            pub trait #value_trait {
                /// What is held per area of cortical type `T`
                type Of<T: #member_trait>;
            }
        }
    }

    /// `{SuperType}SuperTypeMap<V>`, with a map per cortical type from its inner ID to what is held,
    /// and `{SuperType}SuperTypeRef<'a, V>` and `{SuperType}SuperTypeRefMut<'a, V>`, with a variant
    /// per cortical type
    fn generate_super_type_map(&self, super_type: &Ident, template: &SuperTypeTemplate) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let value_trait = self.map_value_trait_name();
        let base_trait = self.base_trait_path();
        let kind = Self::super_type_kind_name(super_type);
        let map = Self::super_type_map_name(super_type);
        let reference = Self::super_type_ref_name(super_type);
        let reference_mut = Self::super_type_ref_mut_name(super_type);

        let names: Vec<&Ident> = template.cortical_types.keys().collect();
        let fields: Vec<Ident> = names.iter().map(|name| Self::map_field_name(name)).collect();
        let attributes: Vec<_> = template.cortical_types.values().map(|cortical_type| &cortical_type.attributes).collect();
        let markers: Vec<proc_macro2::TokenStream> = names.iter().map(|name| self.cortical_type_path(super_type, name)).collect();
        let attributes_ref = &attributes;
        let attributes_ref_mut = &attributes;

        let map_doc = format!("The areas of the `{super_type}` super type of the `{family_name}` family, per cortical type");
        let reference_doc = format!("An area of the `{super_type}` super type of the `{family_name}` family");

        quote! {
            #[doc = #map_doc]
            pub struct #map<V: #value_trait> {
                #(
                    #( #attributes )*
                    pub #fields: ::std::collections::HashMap<<#markers as #base_trait>::Inner, V::Of<#markers>>,
                )*
            }

            impl<V: #value_trait> ::core::default::Default for #map<V> {
                fn default() -> Self {
                    Self {
                        #( #fields: ::core::default::Default::default(), )*
                    }
                }
            }

            impl<V: #value_trait> #map<V> {
                /// How many areas of this super type are held
                pub fn len(&self) -> usize {
                    0 #( + self.#fields.len() )*
                }

                /// If no areas of this super type are held
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
            }

            #[doc = #reference_doc]
            pub enum #reference<'a, V: #value_trait> {
                #(
                    #( #attributes_ref )*
                    #names(&'a V::Of<#markers>),
                )*
            }

            impl<V: #value_trait> #reference<'_, V> {
                /// The cortical type of the area
                pub const fn kind(&self) -> #kind {
                    match self {
                        #( Self::#names(_) => #kind::#names, )*
                    }
                }
            }

            #[doc = #reference_doc]
            pub enum #reference_mut<'a, V: #value_trait> {
                #(
                    #( #attributes_ref_mut )*
                    #names(&'a mut V::Of<#markers>),
                )*
            }

            impl<V: #value_trait> #reference_mut<'_, V> {
                /// The cortical type of the area
                pub const fn kind(&self) -> #kind {
                    match self {
                        #( Self::#names(_) => #kind::#names, )*
                    }
                }
            }
        }
    }

    /// `{Family}Ref<'a, V>` and `{Family}RefMut<'a, V>`, with a variant per super type
    fn generate_family_refs(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let family_kind = self.family_kind_name();
        let value_trait = self.map_value_trait_name();
        let reference = self.ref_name();
        let reference_mut = self.ref_mut_name();

        let names: Vec<&Ident> = self.super_types.keys().collect();
        let attributes: Vec<_> = self.super_types.values().map(|super_type| &super_type.attributes).collect();
        let attributes_mut = &attributes;
        let references = names.iter().map(|name| Self::super_type_ref_name(name));
        let references_mut = names.iter().map(|name| Self::super_type_ref_mut_name(name));
        let doc = format!("An area of the `{family_name}` family, whichever its cortical type");

        quote! {
            #[doc = #doc]
            pub enum #reference<'a, V: #value_trait> {
                #(
                    #( #attributes )*
                    #names(#references<'a, V>),
                )*
            }

            impl<V: #value_trait> #reference<'_, V> {
                /// The cortical type of the area
                pub const fn kind(&self) -> #family_kind {
                    match self {
                        #( Self::#names(inner) => #family_kind::#names(inner.kind()), )*
                    }
                }
            }

            #[doc = #doc]
            pub enum #reference_mut<'a, V: #value_trait> {
                #(
                    #( #attributes_mut )*
                    #names(#references_mut<'a, V>),
                )*
            }

            impl<V: #value_trait> #reference_mut<'_, V> {
                /// The cortical type of the area
                pub const fn kind(&self) -> #family_kind {
                    match self {
                        #( Self::#names(inner) => #family_kind::#names(inner.kind()), )*
                    }
                }
            }
        }
    }

    /// `{Family}Map<V>`, with a field per super type
    fn generate_map(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let identifier = &self.identifier_name;
        let path = self.path_name();
        let value_trait = self.map_value_trait_name();
        let member_trait = self.map_member_trait_name();
        let base_trait = self.base_trait_path();
        let map = self.map_name();
        let reference = self.ref_name();
        let reference_mut = self.ref_mut_name();

        let names: Vec<&Ident> = self.super_types.keys().collect();
        let attributes = self.super_types.values().map(|super_type| &super_type.attributes);
        let fields: Vec<Ident> = names.iter().map(|name| Self::map_field_name(name)).collect();
        let maps = names.iter().map(|name| Self::super_type_map_name(name));

        let leaves = self.cortical_types();
        let patterns: Vec<proc_macro2::TokenStream> = leaves
            .iter()
            .map(|leaf| {
                let (super_type, name) = (leaf.super_type, leaf.name);
                let super_type_path = self.super_type_path_name(super_type);
                quote!(#path::#super_type(#super_type_path::#name(inner)))
            })
            .collect();
        let accesses: Vec<proc_macro2::TokenStream> = leaves
            .iter()
            .map(|leaf| {
                let super_type_field = Self::map_field_name(leaf.super_type);
                let field = Self::map_field_name(leaf.name);
                quote!(self.#super_type_field.#field)
            })
            .collect();
        let wraps = leaves.iter().map(|leaf| {
            let (super_type, name) = (leaf.super_type, leaf.name);
            let super_type_reference = Self::super_type_ref_name(super_type);
            quote!(#reference::#super_type(#super_type_reference::#name(area)))
        });
        let wraps_mut = leaves.iter().map(|leaf| {
            let (super_type, name) = (leaf.super_type, leaf.name);
            let super_type_reference = Self::super_type_ref_mut_name(super_type);
            quote!(#reference_mut::#super_type(#super_type_reference::#name(area)))
        });
        let markers: Vec<proc_macro2::TokenStream> = leaves.iter().map(|leaf| self.cortical_type_path(leaf.super_type, leaf.name)).collect();

        let doc = format!("The areas of the `{family_name}` family, by super type, then cortical type, then inner ID");

        quote! {
            #[doc = #doc]
            pub struct #map<V: #value_trait> {
                #(
                    #( #attributes )*
                    pub #fields: #maps<V>,
                )*
            }

            impl<V: #value_trait> ::core::default::Default for #map<V> {
                fn default() -> Self {
                    Self {
                        #( #fields: ::core::default::Default::default(), )*
                    }
                }
            }

            impl<V: #value_trait> #map<V> {
                /// The area of the given identifier, whichever its cortical type
                pub fn get(&self, identifier: #identifier) -> ::core::option::Option<#reference<'_, V>> {
                    match identifier.path() {
                        #( #patterns => #accesses.get(&inner).map(|area| #wraps), )*
                    }
                }

                /// The area of the given identifier, whichever its cortical type
                pub fn get_mut(&mut self, identifier: #identifier) -> ::core::option::Option<#reference_mut<'_, V>> {
                    match identifier.path() {
                        #( #patterns => #accesses.get_mut(&inner).map(|area| #wraps_mut), )*
                    }
                }

                /// If an area of the given identifier is held
                pub fn contains(&self, identifier: #identifier) -> bool {
                    match identifier.path() {
                        #( #patterns => #accesses.contains_key(&inner), )*
                    }
                }

                /// Every area of cortical type `T`, by its inner ID
                pub fn of<T: #member_trait>(&self) -> &::std::collections::HashMap<<T as #base_trait>::Inner, V::Of<T>> {
                    <T as #member_trait>::areas_of::<V>(self)
                }

                /// Every area of cortical type `T`, by its inner ID
                pub fn of_mut<T: #member_trait>(&mut self) -> &mut ::std::collections::HashMap<<T as #base_trait>::Inner, V::Of<T>> {
                    <T as #member_trait>::areas_of_mut::<V>(self)
                }

                /// The identifier of every area held, grouped by cortical type
                pub fn identifiers(&self) -> impl ::core::iter::Iterator<Item = #identifier> + '_ {
                    ::core::iter::empty()
                        #( .chain(#accesses.keys().map(|inner| #identifier::new::<#markers>(*inner))) )*
                }

                /// How many areas are held
                pub fn len(&self) -> usize {
                    0 #( + self.#fields.len() )*
                }

                /// If no areas are held
                pub fn is_empty(&self) -> bool {
                    self.len() == 0
                }
            }
        }
    }

    /// `{Family}MapMember`, implemented for every cortical type, finding its map within `{Family}Map`
    fn generate_map_member_trait(&self) -> proc_macro2::TokenStream {
        let family_name = &self.family_name;
        let member = self.member_trait_name();
        let member_trait = self.map_member_trait_name();
        let value_trait = self.map_value_trait_name();
        let base_trait = self.base_trait_path();
        let map = self.map_name();

        let impls = self.cortical_types().into_iter().map(|leaf| {
            let marker = self.cortical_type_path(leaf.super_type, leaf.name);
            let super_type_field = Self::map_field_name(leaf.super_type);
            let field = Self::map_field_name(leaf.name);
            quote! {
                impl #member_trait for #marker {
                    fn areas_of<V: #value_trait>(map: &#map<V>) -> &::std::collections::HashMap<<Self as #base_trait>::Inner, V::Of<Self>> {
                        &map.#super_type_field.#field
                    }

                    fn areas_of_mut<V: #value_trait>(map: &mut #map<V>) -> &mut ::std::collections::HashMap<<Self as #base_trait>::Inner, V::Of<Self>> {
                        &mut map.#super_type_field.#field
                    }
                }
            }
        });

        let doc = format!("A cortical type of the `{family_name}` family, which can be found within a [`{map}`]");

        quote! {
            #[doc = #doc]
            pub trait #member_trait: #member + ::core::marker::Sized {
                /// Every area of this cortical type, by its inner ID
                fn areas_of<V: #value_trait>(map: &#map<V>) -> &::std::collections::HashMap<<Self as #base_trait>::Inner, V::Of<Self>>;

                /// Every area of this cortical type, by its inner ID
                fn areas_of_mut<V: #value_trait>(map: &mut #map<V>) -> &mut ::std::collections::HashMap<<Self as #base_trait>::Inner, V::Of<Self>>;
            }

            #( #impls )*
        }
    }
}

//endregion
