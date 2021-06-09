use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields};

pub fn expand_config_parsers(input: DeriveInput) -> syn::Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let parsers = fields
        .into_iter()
        .map(|f| {
            let field_name_str = f.ident.as_ref().unwrap().to_string();
            let field_name = f.ident;

            Ok(quote! {
                #field_name_str => {
                    self.#field_name.parse(value);
                }
            })
        })
        .collect::<syn::Result<TokenStream>>()?;

    let st_name = input.ident;
    Ok(quote! {
        #[automatically_derived]
        impl config_parser::ConfigParser for #st_name {
            fn parse(&mut self, value: toml::Value) {
                match value {
                    toml::Value::Table(table) => {
                       table.into_iter().for_each(|(key, value)| {
                           match key.as_str() {
                               #parsers
                               _ => {}
                           };
                       });
                    }
                    _ => {}
                }
            }
        }
    })
}
