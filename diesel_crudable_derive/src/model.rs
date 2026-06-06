use darling::{
    Error, FromDeriveInput, FromField, Result,
    ast::{self},
    util::PathList,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, parse_macro_input};

use crate::utils::trait_path;
#[derive(FromDeriveInput)]
#[darling(attributes(diesel), allow_unknown_fields, supports(struct_named))]
struct ModelInput {
    ident: syn::Ident,
    generics: syn::Generics,
    #[darling(default)]
    primary_key: PathList,
    data: ast::Data<(), ModelField>,
}

#[derive(FromField)]
struct ModelField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub fn model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    expand_derive_db_model(&ast)
        .unwrap_or_else(|e| e.with_span(&ast).write_errors())
        .into()
}

fn expand_derive_db_model(ast: &DeriveInput) -> Result<TokenStream> {
    let input = ModelInput::from_derive_input(ast)?;
    let path = trait_path("Model")?;
    let name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let fields = input.data.take_struct().unwrap().fields;

    let pk_idents: Vec<Ident> = if input.primary_key.is_empty() {
        vec![format_ident!("id")]
    } else {
        input
            .primary_key
            .iter()
            .map(|p| {
                p.get_ident().cloned().ok_or_else(|| {
                    Error::custom("primary key entries must be simple field names").with_span(p)
                })
            })
            .collect::<Result<_>>()?
    };

    let id_types = pk_idents
        .iter()
        .map(|id| {
            fields
                .iter()
                .find(|f| f.ident.as_ref() == Some(id))
                .map(|f| &f.ty)
                .ok_or_else(|| {
                    Error::custom(format!("primary key field `{id}` not found on `{name}`"))
                        .with_span(id)
                })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl #impl_generics #path for #name #type_generics #where_clause {
            type IdType = (#(#id_types), *);
        }
    })
}
