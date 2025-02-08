// use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, Parser},
    parse_macro_input,
    spanned::Spanned,
    Expr, ItemFn, Meta, Token, Type,
};

#[derive(Clone, Copy)]
enum Macro<'a> {
    Bench { fn_sig: &'a syn::Signature },
    BenchGroup,
}

impl Macro<'_> {
    fn name(&self) -> &'static str {
        match self {
            Self::Bench { .. } => "bench",
            Self::BenchGroup => "bench_group",
        }
    }
}

/// Values from parsed options shared between `#[divan::bench]` and
/// `#[divan::bench_group]`.
///
/// The `crate` option is not included because it is only needed to get proper
/// access to `__private`.
#[derive(Default)]
struct AttrOptions {
    types: bool,
    crate_: bool,
    other_args: Vec<Meta>,
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
                "types" => {
                    attr_options.types = true;
                    return Err(meta.error("TYPES LUL"));
                }
                "crate" => {
                    attr_options.crate_ = true;
                }

                _ => {
                    todo!()
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

#[proc_macro_attribute]
pub fn bench_compat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let attr_options = match AttrOptions::parse(attr) {
        Ok(attr_options) => attr_options,
        Err(error) => return error,
    };
    // let parsed_args = MyBenchArgs::parse(attr).expect("Failed to parse arguments");
    if attr_options.types {
        return quote! {
            compile_error!("`types` argument is not yet supported with codspeed_divan_compat");
        }
        .into();
    }

    let mut filtered_args = Vec::<Meta>::new();

    // if parsed_args.crate_ {
    //     return quote! {
    //         compile_error!("`crate` argument is not yet supported with codspeed_divan_compat adsfljasdlkf");
    //     }
    //     .into();
    // }
    // for arg in parsed_args.other_args {
    //     match &arg {
    //         NameValue(MetaNameValue { path, .. }) => {
    //             if path.is_ident("crate") {
    //                 return quote! {
    //                     compile_error!("`crate` argument is not supported with codspeed_divan_compat");
    //                 }.
    //                 into();
    //             }
    //
    //             if path.is_ident("types") {
    //                 return quote! {
    //                     compile_error!("`type` argument is not yet supported with codspeed_divan_compat");
    //                 }
    //                 .into();
    //             }
    //
    //             if path.is_ident("min_time")
    //                 || path.is_ident("max_time")
    //                 || path.is_ident("sample_size")
    //                 || path.is_ident("sample_count")
    //                 || path.is_ident("skip_ext_time")
    //             {
    //                 // These arguments are ignored in instrumented mode
    //                 continue;
    //             }
    //
    //             filtered_args.push(arg);
    //         }
    //         _ => filtered_args.push(arg),
    //     }
    // }

    let codspeed_divan_crate_ident = format_ident!(
        "{}",
        crate_name("codspeed-divan-compat")
            .map(|found_crate| match found_crate {
                FoundCrate::Itself => "crate".to_string(),
                FoundCrate::Name(name) => name,
            })
            .unwrap_or("codspeed_divan_compat".to_string())
    );

    filtered_args.push(syn::parse_quote!(crate = ::#codspeed_divan_crate_ident));
    // Important: keep macro name in sync with re-exported macro name in divan-compat lib
    let expanded = quote! {
        #[::#codspeed_divan_crate_ident::bench_original(#(#filtered_args),*)]
        #input
    };

    TokenStream::from(expanded)
}

/// Options for generic functions.
#[derive(Default)]
struct GenericOptions {
    /// Generic types over which to instantiate benchmark functions.
    pub types: Option<GenericTypes>,

    /// `const` array/slice over which to instantiate benchmark functions.
    pub consts: Option<Expr>,
}

impl GenericOptions {
    /// Returns `true` if set exclusively to either:
    /// - `types = []`
    /// - `consts = []`
    pub fn is_empty(&self) -> bool {
        match (&self.types, &self.consts) {
            (Some(types), None) => types.is_empty(),
            (None, Some(Expr::Array(consts))) => consts.elems.is_empty(),
            _ => false,
        }
    }

    /// Returns an iterator of multiple `Some` for types, or a single `None` if
    /// there are no types.
    pub fn types_iter(&self) -> Box<dyn Iterator<Item = Option<&dyn ToTokens>> + '_> {
        match &self.types {
            None => Box::new(std::iter::once(None)),
            Some(GenericTypes::List(types)) => {
                Box::new(types.iter().map(|t| Some(t as &dyn ToTokens)))
            }
        }
    }
}

/// Generic types over which to instantiate benchmark functions.
enum GenericTypes {
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

impl GenericTypes {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::List(list) => list.is_empty(),
        }
    }
}
