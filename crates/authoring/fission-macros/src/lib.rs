use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Action)] 
pub fn derive_action(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let action_id_static_name = format_ident!("{}_ACTION_ID", name.to_string().to_uppercase());

    let full_path_str = quote! { concat!(module_path!(), "::", stringify!(#name)) };

    let expanded = quote! {
        #[automatically_derived]
        #[allow(non_upper_case_globals)] 
        lazy_static::lazy_static! {
            pub static ref #action_id_static_name: ::fission_core::action::ActionId = ::fission_core::action::ActionId::from_name(#full_path_str);
        }

        #[automatically_derived]
        impl #impl_generics ::fission_core::action::Action for #name #ty_generics #where_clause {
            fn static_id() -> ::fission_core::action::ActionId {
                *#action_id_static_name 
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(Widget, attributes(widget))]
pub fn derive_widget(_input: TokenStream) -> TokenStream {
    quote!().into()
}
