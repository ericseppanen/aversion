//! ## versioned-macros: macro for deriving the `Versioned` trait.
//!

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Information extracted from the name of a struct.
struct NameInfo {
    struct_name: Ident,
    struct_base: Ident,
    struct_version: u16,
}

impl NameInfo {
    fn from_name(ident: &Ident) -> Self {
        let struct_name = ident.clone();
        let struct_name_string = struct_name.to_string();

        // Split the struct into base and version fields
        let (struct_base, struct_version) = match struct_name_string.rsplit_once('V') {
            Some((base, version)) => {
                if base.len() == 0 {
                    panic!("failed to parse struct name");
                }
                let base = Ident::new(&base, ident.span());
                let version: u16 = version.parse().expect("failed to parse struct version");
                (base, version)
            }
            None => panic!("failed to parse struct name into base+version"),
        };

        NameInfo {
            struct_name,
            struct_base,
            struct_version,
        }
    }
}

fn versioned_name(base: &Ident, version: u16) -> Ident {
    let name = format!("{}V{}", base, version);
    Ident::new(&name, base.span())
}

/// Derive the `Versioned` trait on a struct.
///
#[proc_macro_derive(Versioned)]
pub fn derive_versioned(input: TokenStream) -> TokenStream {
    // parse the input into a DeriveInput syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let NameInfo {
        struct_name,
        struct_base,
        struct_version,
    } = NameInfo::from_name(&input.ident);

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

/// Derive the `UpgradeLatest` trait on a struct.
///
/// It is assumed that all versions 1..N exist, i.e. if `UpgradeLatest`
/// is implemented for `FooV3`, that `FooV2` and `FooV1` both exist and
/// implement `Versioned`.
///
/// It is further assumed that a type alias `Foo` exists and is equivalent
/// to the latest version. In other words: `type Foo = FooV3`
///
#[proc_macro_derive(UpgradeLatest)]
pub fn derive_upgrade_latest(input: TokenStream) -> TokenStream {
    // parse the input into a DeriveInput syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let NameInfo {
        struct_name,
        struct_base,
        struct_version,
    } = NameInfo::from_name(&input.ident);

    // The original generic parameters from the input struct
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Create a list of (version, StructVx), one for each version between 1 and this.
    let mut all_versions = vec![];
    for ii in 1..=struct_version {
        let tup = (ii, versioned_name(&struct_base, ii));
        all_versions.push(tup);
    }

    // Generate the match arm tokens for each version.
    let read_message_arms = all_versions
        .iter()
        .map(|(v, n)| quote_read_message_arm(*v, n, &struct_name));

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
            impl #impl_generics _versioned::group::UpgradeLatest
            for #struct_name #ty_generics #where_clause {

                fn upgrade_latest<Src>(src: &mut Src, ver: u16) -> Result<Self, Src::Error>
                where
                    Src: _versioned::group::DataSource,
                {
                    match ver {
                        #(#read_message_arms)*

                        _ => Err(src.unknown_version::<#struct_base>(ver)),
                    }
                }


            }
        };
    };
    // proc_macro2::TokenStream -> proc_macro::TokenStream
    expanded.into()
}

fn quote_read_message_arm(
    version: u16,
    versioned_name: &Ident,
    target_name: &Ident,
) -> proc_macro2::TokenStream {
    quote! {
        #version => {
            let msg = src.read_message::<#versioned_name>()?;
            let upgraded = <#target_name as _versioned::FromVersion::<#versioned_name>>::from_version(msg);
            Ok(upgraded)
        }
    }
}
