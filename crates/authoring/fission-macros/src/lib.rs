//! Procedural macros for the Fission UI framework.
//!
//! Provides:
//! - `#[derive(Action)]` to generate `Action` trait implementations
//! - `#[fission_action]` to inject the standard Fission action derives
//! - `#[derive(Widget)]` (currently a no-op placeholder)

use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, Attribute, DeriveInput,
    Item, LitStr, Meta, Path, Token,
};

/// Derives the `Action` trait for a struct.
///
/// Generates:
/// 1. An `impl Action for <Name>`.
/// 2. A lazily initialized action ID computed from the fully qualified type path.
///
/// # Requirements
///
/// - The struct should derive `Serialize` and `Deserialize` for dispatch.
#[proc_macro_derive(Action)]
pub fn derive_action(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let full_path_str = quote! { concat!(module_path!(), "::", stringify!(#name)) };
    let fission_core_path = fission_core_path();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics #fission_core_path::action::Action for #name #ty_generics #where_clause {
            fn static_id() -> #fission_core_path::action::ActionId {
                static ACTION_ID: ::std::sync::OnceLock<#fission_core_path::action::ActionId> = ::std::sync::OnceLock::new();
                *ACTION_ID.get_or_init(|| #fission_core_path::action::ActionId::from_name(#full_path_str))
            }
        }
    };

    expanded.into()
}

/// Injects the standard Fission action derives onto a struct or enum.
///
/// By default this adds:
///
/// - `fission_macros::Action`
/// - `serde::Serialize`
/// - `serde::Deserialize`
/// - `Debug`
/// - `Clone`
/// - `PartialEq`
/// - `Eq`
///
/// Use `#[fission_action(no_eq)]` for payloads that cannot implement `Eq`.
#[proc_macro_attribute]
pub fn fission_action(attr: TokenStream, item: TokenStream) -> TokenStream {
    let include_eq = match parse_fission_action_args(attr) {
        Ok(include_eq) => include_eq,
        Err(error) => return error.to_compile_error().into(),
    };

    let item = parse_macro_input!(item as Item);

    let expanded = match item {
        Item::Struct(mut item_struct) => {
            if let Err(error) = merge_action_derives(&mut item_struct.attrs, include_eq) {
                return error.to_compile_error().into();
            }
            quote! { #item_struct }
        }
        Item::Enum(mut item_enum) => {
            if let Err(error) = merge_action_derives(&mut item_enum.attrs, include_eq) {
                return error.to_compile_error().into();
            }
            quote! { #item_enum }
        }
        other => {
            return syn::Error::new_spanned(
                other,
                "#[fission_action] can only be applied to a struct or enum",
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(expanded)
}

/// Reserved derive macro for future widget code generation. Currently a no-op.
#[proc_macro_derive(Widget, attributes(widget))]
pub fn derive_widget(_input: TokenStream) -> TokenStream {
    quote!().into()
}

fn parse_fission_action_args(attr: TokenStream) -> syn::Result<bool> {
    let parser = Punctuated::<Path, Token![,]>::parse_terminated;
    let args = parser.parse(attr)?;
    let mut include_eq = true;

    for arg in args {
        if arg.is_ident("no_eq") {
            include_eq = false;
        } else {
            return Err(syn::Error::new_spanned(
                arg,
                "unsupported #[fission_action(...)] option; supported: no_eq",
            ));
        }
    }

    Ok(include_eq)
}

fn merge_action_derives(attrs: &mut Vec<Attribute>, include_eq: bool) -> syn::Result<()> {
    let mut existing = std::collections::BTreeSet::new();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("derive")) {
        let derives = attr.parse_args_with(Punctuated::<Path, Token![,]>::parse_terminated)?;
        for derive in derives {
            if let Some(segment) = derive.segments.last() {
                existing.insert(segment.ident.to_string());
            }
        }
    }

    let standard_derives: Vec<Path> = vec![
        action_derive_path(),
        serde_derive_path("Serialize"),
        serde_derive_path("Deserialize"),
        parse_quote!(::core::fmt::Debug),
        parse_quote!(::core::clone::Clone),
        parse_quote!(::core::cmp::PartialEq),
    ];
    let mut missing: Vec<Path> = standard_derives
        .into_iter()
        .filter(|path| {
            path.segments
                .last()
                .map(|segment| !existing.contains(&segment.ident.to_string()))
                .unwrap_or(true)
        })
        .collect();

    if include_eq && !existing.contains("Eq") {
        missing.push(parse_quote!(::core::cmp::Eq));
    }

    if !missing.is_empty() {
        attrs.insert(0, parse_quote!(#[derive(#(#missing),*)]));
    }

    if let Some(crate_path) = fission_serde_crate_path() {
        ensure_serde_crate_attr(attrs, &crate_path)?;
    }

    Ok(())
}

fn action_derive_path() -> Path {
    if let Ok(found) = crate_name("fission") {
        return match found {
            FoundCrate::Itself => parse_quote!(::fission::macros::Action),
            FoundCrate::Name(name) => {
                let crate_ident = format_ident!("{}", name);
                parse_quote!(::#crate_ident::macros::Action)
            }
        };
    }

    if let Ok(found) = crate_name("fission-macros") {
        return match found {
            FoundCrate::Itself => parse_quote!(crate::Action),
            FoundCrate::Name(name) => {
                let crate_ident = format_ident!("{}", name);
                parse_quote!(::#crate_ident::Action)
            }
        };
    }

    parse_quote!(Action)
}

fn fission_core_path() -> Path {
    if let Ok(found) = crate_name("fission") {
        return match found {
            FoundCrate::Itself => parse_quote!(::fission::core),
            FoundCrate::Name(name) => {
                let crate_ident = format_ident!("{}", name);
                parse_quote!(::#crate_ident::core)
            }
        };
    }

    if let Ok(found) = crate_name("fission-core") {
        return match found {
            FoundCrate::Itself => parse_quote!(::fission_core),
            FoundCrate::Name(name) => {
                let crate_ident = format_ident!("{}", name);
                parse_quote!(::#crate_ident)
            }
        };
    }

    parse_quote!(fission_core)
}

fn serde_derive_path(derive_name: &str) -> Path {
    let derive_ident = format_ident!("{}", derive_name);

    if let Ok(found) = crate_name("fission") {
        return match found {
            FoundCrate::Itself => parse_quote!(::fission::serde::#derive_ident),
            FoundCrate::Name(name) => {
                let crate_ident = format_ident!("{}", name);
                parse_quote!(::#crate_ident::serde::#derive_ident)
            }
        };
    }

    if let Ok(found) = crate_name("serde") {
        return match found {
            FoundCrate::Itself => parse_quote!(::serde::#derive_ident),
            FoundCrate::Name(name) => {
                let crate_ident = format_ident!("{}", name);
                parse_quote!(::#crate_ident::#derive_ident)
            }
        };
    }

    parse_quote!(serde::#derive_ident)
}

fn fission_serde_crate_path() -> Option<String> {
    crate_name("fission").ok().map(|found| match found {
        FoundCrate::Itself => "::fission::serde".to_string(),
        FoundCrate::Name(name) => format!("::{name}::serde"),
    })
}

fn ensure_serde_crate_attr(attrs: &mut Vec<Attribute>, crate_path: &str) -> syn::Result<()> {
    if has_serde_crate_attr(attrs)? {
        return Ok(());
    }

    let crate_path = LitStr::new(crate_path, proc_macro2::Span::call_site());
    let insert_index = attrs
        .iter()
        .position(|attr| attr.path().is_ident("derive"))
        .map(|index| index + 1)
        .unwrap_or(0);
    attrs.insert(insert_index, parse_quote!(#[serde(crate = #crate_path)]));
    Ok(())
}

fn has_serde_crate_attr(attrs: &[Attribute]) -> syn::Result<bool> {
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("serde")) {
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            if meta.path().is_ident("crate") {
                return Ok(true);
            }
        }
    }

    Ok(false)
}
