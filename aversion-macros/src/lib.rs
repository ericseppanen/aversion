//! ## aversion-macros: macros for deriving the `Versioned` trait.
//!

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Path, Variant};

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
            extern crate aversion as _aversion;

            #[automatically_derived]
            impl #impl_generics _aversion::Versioned
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
    let all_versions = (1..=struct_version)
        .into_iter()
        .map(|ii| (ii, versioned_name(&struct_base, ii)))
        .collect::<Vec<_>>();

    // Generate the match arm tokens for each version.
    let read_message_arms = all_versions
        .iter()
        .map(|(v, n)| quote_read_message_arm(*v, n, &struct_name));

    // Generate the FromVersion impls that skip intermediate versions,
    // and jump directly to the latest.
    let all_hops = (1..struct_version - 1)
        .into_iter()
        .map(|ii| quote_from_version_hop(&struct_base, ii, struct_version))
        .collect::<Vec<_>>();

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
            extern crate aversion as _aversion;

            #[automatically_derived]
            impl #impl_generics _aversion::group::UpgradeLatest
            for #struct_name #ty_generics #where_clause {

                fn upgrade_latest<Src>(src: &mut Src, ver: u16) -> Result<Self, Src::Error>
                where
                    Src: _aversion::group::DataSource,
                {
                    match ver {
                        #(#read_message_arms)*

                        _ => Err(src.unknown_version::<#struct_base>(ver)),
                    }
                }
            }

            #(#all_hops)*
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
            let upgraded = <#target_name as _aversion::FromVersion::<#versioned_name>>::from_version(msg);
            Ok(upgraded)
        }
    }
}

/// Chain FromVersion implementations to skip directly to the latest version.
///
/// If there is a FooV1..FooV4, and there is a FromVersion for each N to N+1,
/// generate the code for `FromVersion<FooV1> for FooV4`.
///
fn quote_from_version_hop(base: &Ident, lo: u16, hi: u16) -> proc_macro2::TokenStream {
    assert!(hi > lo);
    if hi - lo < 2 {
        // The user should already have provided FromVersion<___N> for ___M
        return quote! {};
    }

    // Create identifiers like `v1`, `v2`, etc.
    fn tmp_ident(x: u16) -> Ident {
        format_ident!("v{}", x)
    }

    // Create a chain of upgrades.
    let upgrade_chain = (lo..hi)
        .into_iter()
        .map(|ii| {
            let jj = ii + 1;
            let tmp_ii = tmp_ident(ii);
            let tmp_jj = tmp_ident(jj);
            let ident_jj = versioned_name(base, jj);
            quote! {
                let #tmp_jj = #ident_jj::from_version(#tmp_ii);
            }
        })
        .collect::<Vec<_>>();

    let lo_ident = versioned_name(base, lo);
    let hi_ident = versioned_name(base, hi);
    let lo_tmp = tmp_ident(lo);
    let hi_tmp = tmp_ident(hi);

    quote! {
        impl FromVersion<#lo_ident> for #hi_ident {
            fn from_version(#lo_tmp: #lo_ident) -> Self {
                #(#upgrade_chain)*
                #hi_tmp
            }
        }
    }
}

/// Derive the `GroupDeserialize` trait on a struct.
///
/// This macro expects an enum as input, where each variant contains exactly
/// one field: a type that implements `Versioned + MessageId`.
///
#[proc_macro_derive(GroupDeserialize)]
pub fn derive_group_deserialize(input: TokenStream) -> TokenStream {
    // parse the input into a DeriveInput syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    // The original generic parameters from the input struct
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let source_struct = input.data;
    let variants = if let syn::Data::Enum(syn::DataEnum { variants, .. }) = source_struct {
        variants
    } else {
        panic!("couldn't find enum variants");
    };

    let match_arms = variants
        .iter()
        .map(|v| {
            // Extract basic data about this variant
            let gv = GroupVariant::from_enum_variant(v);
            // Write the GroupDeserialize match arm for this variant
            gv.to_match_arm(enum_name)
        })
        .collect::<Vec<_>>();

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
            extern crate aversion as _aversion;

            #[automatically_derived]
            impl #impl_generics _aversion::GroupDeserialize
            for #enum_name #ty_generics #where_clause {
                fn read_message<Src>(src: &mut Src) -> Result<Self, Src::Error>
                where
                    Src: DataSource,
                {
                    let header = src.read_header()?;
                    match header.msg_id() {
                        #(#match_arms)*
                        _ => {
                            Err(src.unknown_message(header.msg_id()))
                        }
                    }
                }
            }
        };
    };

    // proc_macro2::TokenStream -> proc_macro::TokenStream
    expanded.into()
}

#[derive(Debug)]
struct GroupVariant {
    name: Ident,
    target: Path,
}

impl GroupVariant {
    fn from_enum_variant(variant: &Variant) -> GroupVariant {
        let name = variant.ident.clone();
        let mut target_path: Option<Path> = None;
        if let syn::Fields::Unnamed(syn::FieldsUnnamed {
            unnamed: variant_fields,
            ..
        }) = &variant.fields
        {
            if variant_fields.len() != 1 {
                panic!("enum must contain exactly 1 field");
            }
            let field = variant_fields.first().unwrap();

            if let syn::Type::Path(syn::TypePath { path, .. }) = &field.ty {
                target_path = Some(path.clone());
            }
        }

        let target = target_path
            .unwrap_or_else(|| panic!("failed to extract enum target path for {}", name));

        GroupVariant { name, target }
    }

    fn to_match_arm(&self, enum_name: &Ident) -> proc_macro2::TokenStream {
        let enum_variant = &self.name;
        let struct_name = &self.target;

        quote! {
            #struct_name::MSG_ID => {
                let msg = #struct_name::upgrade_latest(src, header.msg_ver())?;
                Ok(#enum_name::#enum_variant(msg))
            }
        }
    }
}
