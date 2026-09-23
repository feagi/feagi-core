use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{braced, bracketed, parse_macro_input, Expr, ExprLit, ExprUnary, Ident, Lit, LitByteStr, LitInt, LitStr, Result, Token, UnOp};

mod kw {
    syn::custom_keyword!(request_response);

    syn::custom_keyword!(read);
    syn::custom_keyword!(create);
    syn::custom_keyword!(edit);
    syn::custom_keyword!(delete);
    syn::custom_keyword!(patch);

    syn::custom_keyword!(description);
    syn::custom_keyword!(request_parameters);
    syn::custom_keyword!(request);
    syn::custom_keyword!(response);
}

enum PathElement {
    /// A set path element name
    Static(String),
    /// A parameter in the path itself
    Parameter(String)

}


struct RequestCategories {
    read: (),
    create: (),
    edit: (),
    delete: (),
    patch: (),
}



struct RequestCreate {

}
