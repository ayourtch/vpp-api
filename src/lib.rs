extern crate proc_macro;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use proc_macro2::TokenTree;


#[proc_macro_derive(Message,attributes(message_name_and_crc))]
pub fn derive_message(input:proc_macro::TokenStream) -> proc_macro::TokenStream {
        let input = parse_macro_input!(input as DeriveInput);
        let attribute_tokens = input.attrs[0].tokens.clone();
        let mut token_iter = attribute_tokens.into_iter();
        let first = token_iter.next().unwrap();
        let ident =  match first{
            TokenTree::Group(ref g) => {
                let stream = g.stream().clone();
                let mut stream_iter = stream.into_iter();
                stream_iter.next().unwrap().to_string()
            },
            _ => panic!("Wrong format for message name and crc")
        };
        let name = input.ident;
        let expanded = quote! {
                 impl #name {
                    pub fn get_message_name_and_crc() -> String {
                         String::from(#ident)
                    }
                }
            };
        expanded.into()
}