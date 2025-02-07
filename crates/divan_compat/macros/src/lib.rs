// use proc_macro_crate::{crate_name, FoundCrate};

use proc_macro::TokenStream;
// use quote::{format_ident, quote};
use syn::{parse::Parser, parse::Result, parse_macro_input, ItemFn, Meta};

#[derive(Default)]
struct MyBenchArgs {
    // types: bool,
    // crate_: bool,
    // other_args: Vec<Meta>,
}

impl MyBenchArgs {
    fn parse(tokens: TokenStream) -> Result<Self> {
        let mut args = Self::default();

        let attr_parser = syn::meta::parser(|meta| {
            // println!("CALLED");
            // if meta.path.is_ident("types") {
            //     // dbg!("FOUND TYPES");
            //     // dbg!(&meta.path);
            //     // dbg!(&meta.value());
            //     args.types = true;
            // } else if meta.path.is_ident("crate") {
            //     // dbg!("FOUND CRATE");
            //     // dbg!(&meta.path);
            //     // dbg!(&meta.value());
            //     args.crate_ = true;
            // } else {
            //     // // Manually construct `syn::Meta`
            //     // let path = meta.path.clone();
            //     // let parsed_meta = if meta.input.is_empty() {
            //     //     Meta::Path(path)
            //     // } else {
            //     //     let value: syn::Expr = meta.value()?.parse()?;
            //     //     Meta::NameValue(MetaNameValue {
            //     //         path,
            //     //         eq_token: Default::default(),
            //     //         value,
            //     //     })
            //     // };
            //     //
            //     // args.other_args.push(parsed_meta);
            // }
            //
            // println!("SUCCESS");
            Ok(())
        });

        println!("HELLO");
        attr_parser.parse(tokens)?;
        println!("WORLD");
        Ok(args)
    }
}

#[proc_macro_attribute]
pub fn bench_compat(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let parsed_args = MyBenchArgs::parse(attr).expect("Failed to parse arguments");

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

    todo!()
    // let codspeed_divan_crate_ident = format_ident!(
    //     "{}",
    //     crate_name("codspeed-divan-compat")
    //         .map(|found_crate| match found_crate {
    //             FoundCrate::Itself => "crate".to_string(),
    //             FoundCrate::Name(name) => name,
    //         })
    //         .unwrap_or("codspeed_divan_compat".to_string())
    // );
    //
    // filtered_args.push(syn::parse_quote!(crate = ::#codspeed_divan_crate_ident));
    // // Important: keep macro name in sync with re-exported macro name in divan-compat lib
    // let expanded = quote! {
    //     #[::#codspeed_divan_crate_ident::bench_original(#(#filtered_args),*)]
    //     #input
    // };
    //
    // TokenStream::from(expanded)
}
