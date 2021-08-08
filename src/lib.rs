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
        let fields = if let syn::Data::Struct(syn::DataStruct{fields: syn::Fields::Named(syn::FieldsNamed{ref named, .. }), .. }) = input.data {
            named
        } 
        else{
            unimplemented!();
        };
        let option_fields = fields.iter().map(|f|{
            let name = &f.ident; 
            let ty = &f.ty; 
            quote! {#name: std::option::Option<#ty>}
        });
        let builder_init = fields.iter().map(|f|{
            let name = &f.ident; 
            quote! {#name: None}
        });
        let field_methods = fields.iter().map(|f|{
            let name = &f.ident; 
            let ty = &f.ty; 
            quote! {
                fn #name(&mut self, #name:#ty) -> &mut Self{
                    self.#name = Some(#name); 
                    self 
                }
            }

        });
        let build_fields = fields.iter().map(|f|{
            let name = &f.ident; 
            quote! {
                #name: self.#name.clone().ok_or(concat!(stringify!(#name), "is not set"))?
            }
        });
        let builder_ident = syn::Ident::new(&format!("Builder{}",name.to_string()), name.span());
        let expanded = quote! {
                 pub struct #builder_ident{
                     #(#option_fields,)*
                 }
                 impl #builder_ident{
                     #(#field_methods)*
                     fn build(&mut self) -> Result<#name, Box<dyn str::error::Error>>{
                         Ok(#name{
                             #(#build_fields,)*
                        })
                     }
                 }
                 impl #name {
                    pub fn get_message_name_and_crc() -> String {
                         String::from(#ident)
                    }
                    pub fn builder() -> #builder_ident{
                        #builder_ident{
                         #(#builder_init,)*
                        }
                    }
                }
            };
        expanded.into()
}
