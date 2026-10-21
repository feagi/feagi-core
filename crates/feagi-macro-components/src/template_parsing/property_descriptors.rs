use indexmap::IndexMap;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Token};

/// Allows adding comments to a given property. Properties keep their template order
pub struct PropertyDescriptors(IndexMap<Ident, LitStr>);

impl PropertyDescriptors {
    pub fn new_empty() -> Self {
        Self(IndexMap::new())
    }

    pub fn try_get_description(&self, property: &Ident) -> Option<&LitStr> {
        self.0.get(property)
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, Ident, LitStr> {
        self.0.iter()
    }
}

impl Parse for PropertyDescriptors {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field_input;
        braced!(field_input in input);
        let mut properties = IndexMap::new();
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
