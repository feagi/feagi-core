use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{braced, parse_macro_input, Attribute, Expr, Ident, LitByteStr, LitInt, LitStr, Result, Token, Type};

mod kw {
    syn::custom_keyword!(template);

    syn::custom_keyword!(friendly_name);
    syn::custom_keyword!(encoded_data_type);
    syn::custom_keyword!(cortical_id_tag);
    syn::custom_keyword!(io_cortical_areas);

    syn::custom_keyword!(io_cortical_data_type);
    syn::custom_keyword!(relative_position);
    syn::custom_keyword!(channel_dimensions_default);
    syn::custom_keyword!(channel_dimensions_min);
    syn::custom_keyword!(channel_dimensions_max);
    syn::custom_keyword!(io_cortical_generator);
}

/// The full device list
struct CorticalIOUnitTemplateList {
    units: Vec<CorticalIOUnitTemplate>
}

impl Parse for CorticalIOUnitTemplateList {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::template>()?;

        let content;
        braced!(content in input);

        let mut units = Vec::new();
        while !content.is_empty() {
            units.push(content.parse::<CorticalIOUnitTemplate>()?);
        };

        Ok(Self{units})
    }
}


/// An individual definition for a cortical IO unit
struct CorticalIOUnitTemplate {
    //attrs: Vec<Attribute>, // mainly for doc
    unit_name: Ident,
    encoded_data_type: Ident, // TODO better name,
    friendly_name: LitStr,
    cortical_id_tag: LitByteStr, // should be 3 bytes
    io_cortical_areas: Vec<SubunitIOCorticalAreaProperties>
}

impl Parse for CorticalIOUnitTemplate {
    fn parse(input: ParseStream) -> Result<Self> {

        // Comments here describe a Shock Sensor example

        let unit_name: Ident = input.parse()?;

        let content;
        braced!(content in input);

        // encoded_data_type: Percentage,
        content.parse::<kw::encoded_data_type>()?;
        content.parse::<Token![:]>()?;
        let encoded_data_type: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        // friendly_name: "Shock sensor",
        content.parse::<kw::friendly_name>()?;
        content.parse::<Token![:]>()?;
        let friendly_name: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;

        // cortical_id_tag: *b"shk",
        content.parse::<kw::cortical_id_tag>()?;
        content.parse::<Token![:]>()?;
        let cortical_id_tag: LitByteStr = content.parse()?;
        content.parse::<Token![,]>()?;

        // io_cortical areas: {...}
        content.parse::<kw::io_cortical_areas>()?;
        content.parse::<Token![:]>()?;
        let area_content;
        braced!(area_content in input);
        let mut io_cortical_areas = Vec::new();
        while !area_content.is_empty() {
            io_cortical_areas.push(area_content.parse::<SubunitIOCorticalAreaProperties>()?);
        };

        // Optional trailing comma.
        let _ = area_content.parse::<Token![,]>();

        Ok(Self {
            //attrs,
            unit_name,
            encoded_data_type,
            friendly_name,
            cortical_id_tag,
            io_cortical_areas
        })
    }
}



/// The definition for a cortical area under a cortical io unit
struct SubunitIOCorticalAreaProperties {
    /// The position of this cortical area relative to the cortical unit as a whole
    relative_position: Vec<LitInt>, // sint, 3 long
    channel_dimensions_default: Vec<LitInt>, // uint, 3 long, no value may be 0
    channel_dimensions_min: Vec<LitInt>, // uint, 3 long, no value may be 0, each value must be smaller than max
    channel_dimensions_max: Vec<LitInt>, // uint, 3 long, no value may be 0, each value must be bigger than min
    /// The enum defining a cortical area generator
    io_cortical_generator: Expr
}

impl Parse for SubunitIOCorticalAreaProperties {
    fn parse(input: ParseStream) -> Result<Self> {

        let content;
        braced!(content in input);

        content.parse::<kw::io_cortical_data_type>()?;
        content.parse::<Token![:]>()?;
        let io_cortical_data_type: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::relative_position>()?;
        content.parse::<Token![:]>()?;
        let relative_position: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::channel_dimensions_default>()?;
        content.parse::<Token![:]>()?;
        let channel_dimensions_default: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::channel_dimensions_min>()?;
        content.parse::<Token![:]>()?;
        let channel_dimensions_min: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::channel_dimensions_max>()?;
        content.parse::<Token![:]>()?;
        let channel_dimensions_max: Ident = content.parse()?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::io_cortical_generator>()?;
        content.parse::<Token![:]>()?;
        let io_cortical_generator: Expr = content.parse()?;
        content.parse::<Token![,]>()?;

        // Optional trailing comma.
        let _ = content.parse::<Token![,]>();

        Ok(Self{
            relative_position,
            channel_dimensions_default,
            channel_dimensions_min,
            channel_dimensions_max,
            io_cortical_generator,
        })
    }
}

#[proc_macro]
pub fn on_cortical_template(input: TokenStream) -> TokenStream {
    let template_list = parse_macro_input!(input as CorticalIOUnitTemplateList);

    // TODO Validations

    // TODO Collect from token stream to put into macro generation

    let expanded = quote! {
        #[macro_export]
        macro_rules! #macro_name {
            ($consumer:ident) => {
                $consumer! {
                    #(
                        #unit_names {
                            unit_name: #unit_name,
                            encoded_data_type: #encoded_data_type,
                            friendly_name: #friendly_name,
                            cortical_id_tag: #cortical_id_tag,
                            io_cortical_areas: {#(
                                relative_position: #relative_position,
                                channel_dimensions_default: #channel_dimensions_default,
                                channel_dimensions_min: #channel_dimensions_min,
                                channel_dimensions_max: #channel_dimensions_max,
                                io_cortical_generator: #io_cortical_generator
                            )}
                        }
                    )
                }

            }
        }
    };

    expanded.into()
}