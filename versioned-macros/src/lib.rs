//! ## versioned-macros: macro for deriving the `Versioned` trait.
//!

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive the `Versioned` trait on a struct.
///
#[proc_macro_derive(Versioned)]
pub fn derive_versioned(input: TokenStream) -> TokenStream {
    // parse the input into a DeriveInput syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // The name of the struct
    let struct_name = input.ident;
    let struct_name_string = struct_name.to_string();

    // Split the struct into base and version fields
    let (struct_base, struct_version) = match struct_name_string.rsplit_once('V') {
        Some((base, version)) => {
            if base.len() == 0 {
                panic!("failed to parse struct name");
            }
            let base_name = format!("{}Base", base);
            let base = Ident::new(&base_name, struct_name.span());
            let version: u16 = version.parse().expect("failed to parse struct version");
            (base, version)
        }
        None => panic!("failed to parse struct name into base+version"),
    };

    // The original generic parameters from the input struct
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        #[doc(hidden)]
        #[allow(
            non_upper_case_globals,
            unused_attributes,
            unused_qualifications,
            non_camel_case_types,
            non_snake_case
        )]
        const _: () = {
            #[allow(rust_2018_idioms, clippy::useless_attribute)]
            extern crate versioned as _versioned;

            #[automatically_derived]
            impl #impl_generics _versioned::Versioned
            for #struct_name #ty_generics #where_clause {
                const VER: u16 = #struct_version;
                type Base = #struct_base;
            }
        };
    };
    // proc_macro2::TokenStream -> proc_macro::TokenStream
    expanded.into()
}
