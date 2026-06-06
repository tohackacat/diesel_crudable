mod marker;
mod model;
mod utils;

use marker::marker;
use model::model;

use proc_macro::TokenStream;

#[proc_macro_derive(Model, attributes(diesel))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    model(input)
}

#[proc_macro_derive(Readable)]
pub fn derive_readable(input: TokenStream) -> TokenStream {
    marker("Readable", input)
}

#[proc_macro_derive(Deletable)]
pub fn derive_deletable(input: TokenStream) -> TokenStream {
    marker("Deletable", input)
}

#[proc_macro_derive(Updatable)]
pub fn derive_updatable(input: TokenStream) -> TokenStream {
    marker("Updatable", input)
}

#[proc_macro_derive(Creatable)]
pub fn derive_creatable(input: TokenStream) -> TokenStream {
    marker("Creatable", input)
}
