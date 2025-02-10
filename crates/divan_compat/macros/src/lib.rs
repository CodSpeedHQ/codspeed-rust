mod args;

use args::AttrOptions;
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Expr, ItemFn, Meta};

#[proc_macro_attribute]
pub fn bench_compat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let attr_options = match AttrOptions::parse(attr) {
        Ok(attr_options) => attr_options,
        Err(error) => return error,
    };

    if attr_options.crate_ {
        return quote! {
            compile_error!("`crate` argument is yet supported with codspeed_divan_compat");
        }
        .into();
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

    let mut transfered_args = attr_options.other_args;

    transfered_args.push(syn::parse_quote!(crate = ::#codspeed_divan_crate_ident));

    if let Some(types) = attr_options.types {
        transfered_args.push(Meta::NameValue(syn::MetaNameValue {
            path: syn::parse_quote!(types),
            eq_token: Default::default(),
            value: Expr::Verbatim(types.into_token_stream()),
        }));
    }

    // WARN: keep macro name in sync with re-exported macro name in divan-compat lib
    let expanded = quote! {
        #[::#codspeed_divan_crate_ident::bench_original(#(#transfered_args),*)]
        #input
    };

    TokenStream::from(expanded)
}
