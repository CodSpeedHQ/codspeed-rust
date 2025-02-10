use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    Expr, Meta, MetaNameValue, Token, Type,
};

/// Values from parsed options shared between `#[divan::bench]` and
/// `#[divan::bench_group]`.
///
/// The `crate` option is not included because it is only needed to get proper
/// access to `__private`.
#[derive(Default)]
pub(crate) struct AttrOptions {
    pub(crate) types: Option<GenericTypes>,
    pub(crate) crate_: bool,
    pub(crate) other_args: Vec<Meta>,
}

#[allow(unreachable_code)]
impl AttrOptions {
    pub fn parse(tokens: TokenStream) -> Result<Self, TokenStream> {
        let mut attr_options = Self::default();

        let attr_parser = syn::meta::parser(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(meta.error("Unexpected attribute"));
            };

            let ident_name = ident.to_string();
            let ident_name = ident_name.strip_prefix("r#").unwrap_or(&ident_name);

            match ident_name {
                // Divan accepts type syntax that is not parseable into syn::Meta out of the box,
                // so we parse and rebuild the arguments manually.
                "types" => {
                    attr_options.types = Some(meta.value()?.parse()?);
                }
                "crate" => {
                    attr_options.crate_ = true;
                    meta.value()?.parse::<Expr>()?; // Discard the value
                }
                "min_time" | "max_time" | "sample_size" | "sample_count" | "skip_ext_time" => {
                    // These arguments are ignored for codspeed runs
                    meta.value()?.parse::<Expr>()?; // Discard the value
                }
                _ => {
                    let path = meta.path.clone();
                    let parsed_meta = if meta.input.is_empty() {
                        Meta::Path(path)
                    } else {
                        let value: syn::Expr = meta.value()?.parse()?;
                        Meta::NameValue(MetaNameValue {
                            path,
                            eq_token: Default::default(),
                            value: Expr::Verbatim(value.into_token_stream()),
                        })
                    };

                    attr_options.other_args.push(parsed_meta);
                }
            }

            Ok(())
        });

        match attr_parser.parse(tokens) {
            Ok(()) => {}
            Err(error) => return Err(error.into_compile_error().into()),
        }

        Ok(attr_options)
    }
}

/// Generic types over which to instantiate benchmark functions.
pub(crate) enum GenericTypes {
    /// List of types, e.g. `[i32, String, ()]`.
    List(Vec<proc_macro2::TokenStream>),
}

impl Parse for GenericTypes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);

        Ok(Self::List(
            content
                .parse_terminated(Type::parse, Token![,])?
                .into_iter()
                .map(|ty| ty.into_token_stream())
                .collect(),
        ))
    }
}

impl ToTokens for GenericTypes {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::List(list) => {
                let type_tokens = list.iter().cloned().map_into::<proc_macro2::TokenStream>();
                tokens.extend(quote! { [ #(#type_tokens),* ] });
            }
        }
    }
}
