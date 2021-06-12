use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parser;
use parser::expand_config_parsers;

#[proc_macro_derive(ConfigParse)]
pub fn parsers(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_config_parsers(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
