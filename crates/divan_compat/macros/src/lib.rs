use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::Parse,
    parse_macro_input,
    punctuated::Punctuated,
    ItemFn,
    Meta::{self, NameValue},
    MetaNameValue, Token,
};

struct MyBenchArgs {
    args: Punctuated<Meta, Token![,]>,
}

impl Parse for MyBenchArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            args: Punctuated::parse_terminated(input)?,
        })
    }
}

#[proc_macro_attribute]
pub fn bench_compat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed_args = parse_macro_input!(attr as MyBenchArgs);
    let input = parse_macro_input!(item as ItemFn);

    let mut filtered_args = Vec::new();

    for arg in parsed_args.args {
        match &arg {
            NameValue(MetaNameValue { path, .. }) => {
                if path.is_ident("crate") {
                    return quote! {
                        compile_error!("`crate` argument is not supported with codspeed_divan_compat");
                    }.
                    into();
                }

                if path.is_ident("types") {
                    return quote! {
                        compile_error!("`type` argument is not yet supported with codspeed_divan_compat");
                    }
                    .into();
                }

                if path.is_ident("min_time")
                    || path.is_ident("max_time")
                    || path.is_ident("sample_size")
                    || path.is_ident("sample_count")
                    || path.is_ident("skip_ext_time")
                {
                    // These arguments are ignored in instrumented mode
                    continue;
                }

                filtered_args.push(arg);
            }
            _ => filtered_args.push(arg),
        }
    }

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
