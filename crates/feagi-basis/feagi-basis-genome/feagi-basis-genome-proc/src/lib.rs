use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{braced, bracketed, parse_macro_input, Expr, ExprLit, ExprUnary, Ident, Lit, LitByteStr, LitInt, LitStr, Result, Token, UnOp};

// NOTE: Using standard .expect errors here since this is all compile time stuff

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

/// The full device list.
struct CorticalIOUnitTemplateList {
    units: Vec<CorticalIOUnitTemplate>,
}

impl Parse for CorticalIOUnitTemplateList {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::template>()?;

        let content;
        braced!(content in input);

        let mut units = Vec::new();
        while !content.is_empty() {
            units.push(content.parse::<CorticalIOUnitTemplate>()?);
            let _ = content.parse::<Token![,]>();
        }

        Ok(Self { units })
    }
}

/// An individual definition for a cortical IO unit.
struct CorticalIOUnitTemplate {
    unit_name: Ident,
    /// Identifier to signify what encoder / decoder to use.
    encoded_data_type: Ident,
    friendly_name: LitStr,
    /// Should be 3 bytes.
    cortical_id_tag: LitByteStr,
    io_cortical_areas: Vec<SubunitIOCorticalAreaProperties>,
}

impl Parse for CorticalIOUnitTemplate {
    fn parse(input: ParseStream) -> Result<Self> {
        // Comments here describe a Shock Sensor example.
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

        // io_cortical_areas: { ... }
        content.parse::<kw::io_cortical_areas>()?;
        content.parse::<Token![:]>()?;
        let area_content;
        braced!(area_content in content);
        let mut io_cortical_areas = Vec::new();
        while !area_content.is_empty() {
            io_cortical_areas.push(area_content.parse::<SubunitIOCorticalAreaProperties>()?);
            let _ = area_content.parse::<Token![,]>();
        }

        // Optional trailing comma for the unit body.
        let _ = content.parse::<Token![,]>();

        Ok(Self {
            unit_name,
            encoded_data_type,
            friendly_name,
            cortical_id_tag,
            io_cortical_areas,
        })
    }
}

/// The definition for a cortical area under a cortical IO unit.
struct SubunitIOCorticalAreaProperties {
    /// The enum defining an IO data type for this cortical area.
    io_cortical_data_type: Ident,
    /// Position of this cortical area relative to the cortical unit as a whole.
    relative_position: Vec<Expr>, // sint, 3 long
    channel_dimensions_default: Vec<LitInt>, // uint, 3 long, no value may be 0
    channel_dimensions_min: Vec<LitInt>,     // uint, 3 long, no value may be 0, each value must be smaller or equal than max
    channel_dimensions_max: Vec<LitInt>,     // uint, 3 long, no value may be 0, each value must be bigger or equal than min
    /// The enum or macro invocation defining a cortical area generator.
    io_cortical_generator: Expr,
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
        let relative_position = parse_signed_triplet(&content)?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::channel_dimensions_default>()?;
        content.parse::<Token![:]>()?;
        let channel_dimensions_default = parse_unsigned_triplet(&content)?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::channel_dimensions_min>()?;
        content.parse::<Token![:]>()?;
        let channel_dimensions_min = parse_unsigned_triplet(&content)?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::channel_dimensions_max>()?;
        content.parse::<Token![:]>()?;
        let channel_dimensions_max = parse_unsigned_triplet(&content)?;
        content.parse::<Token![,]>()?;

        content.parse::<kw::io_cortical_generator>()?;
        content.parse::<Token![:]>()?;
        let io_cortical_generator: Expr = content.parse()?;

        // Optional trailing comma.
        let _ = content.parse::<Token![,]>();

        Ok(Self {
            io_cortical_data_type,
            relative_position,
            channel_dimensions_default,
            channel_dimensions_min,
            channel_dimensions_max,
            io_cortical_generator,
        })
    }
}

fn parse_unsigned_triplet(input: ParseStream) -> Result<Vec<LitInt>> {
    let values = parse_litint_array(input)?;
    Ok(values)
}

fn parse_signed_triplet(input: ParseStream) -> Result<Vec<Expr>> {
    let inner;
    bracketed!(inner in input);
    let values: Punctuated<Expr, Token![,]> = inner.parse_terminated(Expr::parse, Token![,])?;
    Ok(values.into_iter().collect())
}

fn parse_litint_array(input: ParseStream) -> Result<Vec<LitInt>> {
    let inner;
    bracketed!(inner in input);
    let values: Punctuated<LitInt, Token![,]> = inner.parse_terminated(LitInt::parse, Token![,])?;
    Ok(values.into_iter().collect())
}

fn expr_to_i64(expr: &Expr) -> Result<i64> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Int(int_lit), .. }) => int_lit.base10_parse::<i64>(),
        Expr::Unary(ExprUnary { op: UnOp::Neg(_), expr, .. }) => match &**expr {
            Expr::Lit(ExprLit { lit: Lit::Int(int_lit), .. }) => int_lit.base10_parse::<i64>().map(|v| -v),
            _ => Err(syn::Error::new_spanned(expr, "expected integer literal after '-'")),
        },
        _ => Err(syn::Error::new_spanned(expr, "expected signed integer literal")),
    }
}

fn validate_unsigned_triplet(values: &[LitInt], field_name: &str) -> Result<()> {
    if values.len() != 3 {
        return Err(syn::Error::new_spanned(
            quote! { [#(#values),*] },
            format!("{field_name} must contain exactly 3 unsigned integers"),
        ));
    }

    for value in values {
        let parsed = value.base10_parse::<u64>()?;
        if parsed == 0 {
            return Err(syn::Error::new(value.span(), format!("{field_name} values must be greater than zero")));
        }
    }

    Ok(())
}

fn validate_template(template_list: &CorticalIOUnitTemplateList) -> Result<()> {
    let mut seen_unit_names = HashSet::<String>::new();
    let mut seen_cortical_id_tags = HashSet::<Vec<u8>>::new();

    for unit in &template_list.units {
        let unit_name = unit.unit_name.to_string();
        if !seen_unit_names.insert(unit_name.clone()) {
            return Err(syn::Error::new(unit.unit_name.span(), format!("duplicate unit name `{unit_name}`")));
        }

        if unit.friendly_name.value().trim().is_empty() {
            return Err(syn::Error::new(unit.friendly_name.span(), "friendly_name must not be empty"));
        }

        let cortical_id_tag = unit.cortical_id_tag.value();
        if cortical_id_tag.len() != 3 {
            return Err(syn::Error::new(unit.cortical_id_tag.span(), "cortical_id_tag must be exactly 3 bytes"));
        }
        if !seen_cortical_id_tags.insert(cortical_id_tag.clone()) {
            return Err(syn::Error::new(
                unit.cortical_id_tag.span(),
                format!("duplicate cortical_id_tag `{}`", String::from_utf8_lossy(&cortical_id_tag)),
            ));
        }

        if unit.io_cortical_areas.is_empty() {
            return Err(syn::Error::new(
                unit.unit_name.span(),
                "io_cortical_areas must contain at least one entry",
            ));
        }

        for area in &unit.io_cortical_areas {
            if area.relative_position.len() != 3 {
                return Err(syn::Error::new(
                    area.relative_position.first().map_or(unit.unit_name.span(), Spanned::span),
                    "relative_position must contain exactly 3 signed integers",
                ));
            }
            for value in &area.relative_position {
                expr_to_i64(value)?;
            }

            validate_unsigned_triplet(&area.channel_dimensions_default, "channel_dimensions_default")?;
            validate_unsigned_triplet(&area.channel_dimensions_min, "channel_dimensions_min")?;
            validate_unsigned_triplet(&area.channel_dimensions_max, "channel_dimensions_max")?;

            for idx in 0..3 {
                let min_v = area.channel_dimensions_min[idx].base10_parse::<u64>()?;
                let max_v = area.channel_dimensions_max[idx].base10_parse::<u64>()?;
                let default_v = area.channel_dimensions_default[idx].base10_parse::<u64>()?;

                if min_v > max_v {
                    return Err(syn::Error::new(
                        area.channel_dimensions_min[idx].span(),
                        format!("channel_dimensions_min[{idx}] must be smaller or equal than channel_dimensions_max[{idx}]"),
                    ));
                }
                if default_v < min_v || default_v > max_v {
                    return Err(syn::Error::new(
                        area.channel_dimensions_default[idx].span(),
                        format!("channel_dimensions_default[{idx}] must be (inclusively) within [min, max] bounds"),
                    ));
                }
            }
        }
    }

    Ok(())
}

#[proc_macro]
/// Generates `for_each_cortical_io_unit_template!` from a `template { ... }` declaration.
///
/// Expected input form:
/// `template {
///     UnitName {
///         encoded_data_type: Kind,
///         friendly_name: "Name",
///         cortical_id_tag: b"abc",
///         io_cortical_areas: {
///             {
///                 io_cortical_data_type: Type,
///                 relative_position: [0, 0, 0],
///                 channel_dimensions_default: [1, 1, 1],
///                 channel_dimensions_min: [1, 1, 1],
///                 channel_dimensions_max: [2, 2, 2],
///                 io_cortical_generator: Generator::Variant,
///             },
///         },
///     },
/// }`
pub fn on_cortical_template(input: TokenStream) -> TokenStream {
    let template_list = parse_macro_input!(input as CorticalIOUnitTemplateList);

    if let Err(error) = validate_template(&template_list) {
        return error.to_compile_error().into();
    }

    let macro_name = format_ident!("for_each_cortical_io_unit_template");

    let unit_tokens = template_list.units.iter().map(|unit| {
        let unit_name = &unit.unit_name;
        let encoded_data_type = &unit.encoded_data_type;
        let friendly_name = &unit.friendly_name;
        let cortical_id_tag = &unit.cortical_id_tag;

        let area_tokens = unit.io_cortical_areas.iter().map(|area| {
            let io_cortical_data_type = &area.io_cortical_data_type;
            let relative_position = &area.relative_position;
            let channel_dimensions_default = &area.channel_dimensions_default;
            let channel_dimensions_min = &area.channel_dimensions_min;
            let channel_dimensions_max = &area.channel_dimensions_max;
            let io_cortical_generator = &area.io_cortical_generator;

            quote! {
                {
                    io_cortical_data_type: #io_cortical_data_type,
                    relative_position: [#(#relative_position),*],
                    channel_dimensions_default: [#(#channel_dimensions_default),*],
                    channel_dimensions_min: [#(#channel_dimensions_min),*],
                    channel_dimensions_max: [#(#channel_dimensions_max),*],
                    io_cortical_generator: #io_cortical_generator,
                }
            }
        });

        quote! {
            #unit_name {
                unit_name: #unit_name,
                encoded_data_type: #encoded_data_type,
                friendly_name: #friendly_name,
                cortical_id_tag: #cortical_id_tag,
                io_cortical_areas: {
                    #(#area_tokens,)*
                },
            }
        }
    });

    let expanded = quote! {
        #[macro_export]
        macro_rules! #macro_name {
            ($consumer:ident) => {
                $consumer! {
                    #(#unit_tokens)*
                }
            };
        }
    };

    expanded.into()
}
