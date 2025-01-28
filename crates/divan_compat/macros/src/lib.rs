use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, ItemFn, Meta, MetaNameValue, Token,
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
            Meta::NameValue(MetaNameValue { path, .. }) if path.is_ident("crate") => {
                return quote! {
                    compile_error!("crate argument is not supported with codspeed_divan_compat");
                }
                .into();
            }
            _ => filtered_args.push(arg),
        }
    }

    filtered_args.push(syn::parse_quote!(crate = ::codspeed_divan_compat));

    // Important: keep macro name in sync with re-exported macro name in divan-compat lib
    let expanded = quote! {
        #[::codspeed_divan_compat::bench_original(#(#filtered_args),*)]
        #input
    };

    TokenStream::from(expanded)
}
