use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn trait_path(trait_name: &str) -> darling::Result<TokenStream> {
    let t = Ident::new(trait_name, Span::call_site());
    Ok(
        match crate_name("diesel_crudable").map_err(|e| {
            darling::Error::custom(format!(
                "derive({trait_name}) needs `diesel_crudable` as a dependency: {e}"
            ))
        })? {
            FoundCrate::Itself => quote! (crate::#t),
            FoundCrate::Name(name) => {
                let krate = Ident::new(&name, Span::call_site());
                quote!(::#krate::#t)
            }
        },
    )
}
