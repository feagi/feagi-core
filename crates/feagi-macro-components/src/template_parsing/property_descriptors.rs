use std::collections::HashMap;
use proc_macro2::{Ident};
use quote::{quote, TokenStreamExt};
use syn::{braced, Lit, LitStr, Token};
use syn::parse::{Parse, ParseStream};
use crate::templates::Unparse;

/// Allows adding comments to a given property
pub struct PropertyDescriptors(HashMap<Ident, LitStr>);

impl PropertyDescriptors {
    pub fn try_get_description(&self, property: &Ident) -> Option<&LitStr> {
        self.0.get(property)
    }
}

impl Parse for PropertyDescriptors {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field_input;
        braced!(field_input in input);
        let mut properties = HashMap::new();
        while !field_input.is_empty() {
            let property = field_input.parse::<Ident>()?;
            field_input.parse::<Token![:]>()?;
            let description = field_input.parse::<LitStr>()?;
            properties.insert(property, description);
            if field_input.is_empty() {
                break;
            }
            field_input.parse::<Token![,]>()?;
        }
        Ok(Self(properties))
    }
}

impl Unparse for PropertyDescriptors {
    fn unparse(&self) -> proc_macro2::TokenStream {
        let mut stream = proc_macro2::TokenStream::new();

        for (prop, desc) in &self.0 {
            stream.extend(quote!(#prop: #desc));
        }

        quote! {
            {
                #stream
            }
        }

    }
}


