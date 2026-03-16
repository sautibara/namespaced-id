use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro]
pub fn ident(attr: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(attr as syn::LitStr);
    let string = lit_str.value();

    match namespaced_id_core::validate::<2>(&string) {
        Ok(()) => quote! { namespaced_id::NamespacedIdRef::from_str_unchecked(#lit_str) }.into(),
        Err(err) => syn::Error::new_spanned(&lit_str, format!("failed to parse id: {err}"))
            .into_compile_error()
            .into(),
    }
}
