use std::collections::HashMap;
use proc_macro2::{Ident};
use quote::quote;
use syn::{braced, LitStr, Token, Type};
use syn::parse::{Parse, ParseStream};
use crate::common::RustVisibility;
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::templates::Unparse;

/// Struct fields as `field_name: Type`, with an optional description string.
///
/// ```text
/// {
///     field_name: Type, "optional description",
/// }
/// ```
#[derive(Clone)]
pub struct StructTemplate(HashMap<Ident, (Type, Option<LitStr>)>);

impl StructTemplate {
    
    pub fn new_empty() -> Self {
        Self(HashMap::new())
    }
    
    /// Returns the type and optional description registered for `field`.
    pub fn try_get_field(&self, field: &Ident) -> Option<&(Type, Option<LitStr>)> {
        self.0.get(field)
    }

    /// Outputs a token stream that generates this definition as a struct, with name, optional comment, visibility, and derives
    pub fn generate_rust_struct(&self, struct_name: Ident, description: Option<LitStr>, derives: &Vec<Ident>, visibility: RustVisibility) -> proc_macro2::TokenStream {
        let mut stream = proc_macro2::TokenStream::new();


        if let Some(comment) = description {
            stream.extend(quote! {
                #[doc = #comment]
            })
        };

        if !derives.is_empty()
        {
            stream.extend(quote! {
                #[derive( #( #derives ),* )]
            })
        };

        let mut properties = proc_macro2::TokenStream::new();
        for (name, (property_type, optional_comment)) in &self.0 {
            if let Some(comment) = optional_comment {
                properties.extend(quote! {
                    #[doc = #comment]
                    #name: #property_type,
                })
            } else {
                properties.extend(quote! {
                    #name: #property_type,
                })
            }
        }


        let visibility = visibility.to_token_stream();
        stream.extend(quote! {
            #visibility struct #struct_name
            {
                #properties
            }
        });
        stream
    }

    /// Tries to find a field by name. If one is found, overwrites its description with the given.
    /// If none is found, does nothing.
    pub fn try_overwrite_description(&mut self, field_name: &Ident, description: Option<LitStr>) {
        let field = self.0.get_mut(field_name);
        if let Some(f) = field {
            f.1 = description
        }
    }

    /// Given a PropertyDescriptors, iterates through it to replace any descriptions of this with
    /// the ones from that of the same name
    pub fn overwrite_descriptions_from_property_descriptors(&mut self, descriptors: &PropertyDescriptors) {
        for (name, description) in descriptors.iter() {
            let field = self.0.get_mut(name);
            if let Some(f) = field {
                f.1 = Some(description.clone())
            }
        }
    }
    
    pub fn insert_property(&mut self, property_name: Ident, property_type: Type, property_description: Option<LitStr>) {
        self.0.insert(property_name, (property_type, property_description));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_map(&self) -> &HashMap<Ident, (Type, Option<LitStr>)> {
        &self.0
    }
}

impl Parse for StructTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field_input;
        braced!(field_input in input);
        let mut fields = HashMap::new();
        while !field_input.is_empty() {
            let field_name = field_input.parse::<Ident>()?;
            field_input.parse::<Token![:]>()?;
            let field_type = field_input.parse::<Type>()?;
            // A comma followed by a string belongs to this field. A comma followed by
            // another field is the entry separator and is consumed after insert.
            let description = if field_input.peek(Token![,]) && field_input.peek2(LitStr) {
                field_input.parse::<Token![,]>()?;
                Some(field_input.parse::<LitStr>()?)
            } else {
                None
            };
            fields.insert(field_name, (field_type, description));
            if field_input.is_empty() {
                break;
            }
            field_input.parse::<Token![,]>()?;
        }
        Ok(Self(fields))
    }
}

impl Unparse for StructTemplate {
    fn unparse(&self) -> proc_macro2::TokenStream {
        let mut stream = proc_macro2::TokenStream::new();

        for (field_name, (field_type, description)) in &self.0 {
            match description {
                Some(description) => {
                    stream.extend(quote!(#field_name: #field_type, #description,));
                }
                None => {
                    stream.extend(quote!(#field_name: #field_type,));
                }
            }
        }

        quote! {
            {
                #stream
            }
        }
    }
}
