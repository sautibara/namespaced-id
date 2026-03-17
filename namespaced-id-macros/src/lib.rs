//! Procedural macros for the `namespaced-id` crate.

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

fn gen_ident<const N: usize>(attr: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(attr as syn::LitStr);
    let string = lit_str.value();

    match namespaced_id_core::validate::<N>(&string) {
        Ok(()) => {
            quote! { namespaced_id::DelimitedIdRef::<#N>::from_str_unchecked(#lit_str) }.into()
        }
        Err(err) => syn::Error::new_spanned(&lit_str, format!("failed to parse id: {err}"))
            .into_compile_error()
            .into(),
    }
}

#[proc_macro]
pub fn ident_component(attr: TokenStream) -> TokenStream {
    gen_ident::<1>(attr)
}

#[proc_macro]
pub fn ident(attr: TokenStream) -> TokenStream {
    gen_ident::<2>(attr)
}

#[proc_macro]
pub fn op_ident(attr: TokenStream) -> TokenStream {
    gen_ident::<3>(attr)
}
