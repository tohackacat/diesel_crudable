use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::utils::trait_path;

pub fn marker(trait_name: &str, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    expand_marker(trait_name, &ast)
        .unwrap_or_else(|e| e.with_span(&ast).write_errors())
        .into()
}

fn expand_marker(trait_name: &str, ast: &DeriveInput) -> darling::Result<TokenStream> {
    let name = &ast.ident;
    let path = trait_path(trait_name)?;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
    if let syn::Data::Struct(_) = ast.data {
        Ok(quote! {impl #impl_generics #path for #name #type_generics #where_clause {} })
    } else {
        Err(darling::Error::custom("this derive can only be applied to structs"))
    }
}
