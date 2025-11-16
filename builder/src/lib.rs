use proc_macro::{Span, TokenStream};
use syn::{
    DeriveInput,
    parse_macro_input,
    Ident,
    Data,
    Fields
};
use quote::{quote};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let builder_name = Ident::new(&format!("{}Builder", name), Span::call_site().into());
    let err_name = Ident::new(&format!("{}BuilderError", name), Span::call_site().into());

    let fields = match input.data {
        Data::Struct(struct_data) => match struct_data.fields {
            Fields::Named(named_fields) => named_fields.named,
            _ => panic!("Expected named fields"),
        },
        _ => panic!("Expected a struct!")
    };

    let field_inits = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            #field_name: None,
        }
    });

    let field_idents = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_ty = &field.ty;

        quote! {
            #field_name: Option<#field_ty>,
        }
    });

    let method_decls = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        quote! {
            pub fn #field_name (&mut self, #field_name: #field_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    });

    let build_decls = fields.iter().map(|field| {
        let field_name = &field.ident;

        quote! {
            #field_name: self.#field_name.take().ok_or(#err_name)?,
        }
    });

    return quote! {
        use std::error::Error;
        use std::fmt;

        impl #name {
            pub fn builder() -> #builder_name {
                return #builder_name {
                    #(#field_inits)*
                }
            }
        }

        pub struct #builder_name {
            #(#field_idents)*
        }

        #[derive(Debug, Clone)]
        pub struct #err_name;

        impl fmt::Display for #err_name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Expected all fields to be defined.")
            }
        }

        impl #builder_name {
            fn build(&mut self) -> Result<#name, #err_name> {
                return Ok(#name {
                    #(#build_decls)*
                });
            }

            #(#method_decls)*
        }
    }.into();
}
