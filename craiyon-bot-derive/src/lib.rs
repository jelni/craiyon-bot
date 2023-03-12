#![warn(clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Data, DataStruct, DeriveInput, Fields};

#[proc_macro_derive(ParseCommand)]
pub fn parse_command_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_parse_command(&ast)
}

fn impl_parse_command(ast: &DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    let field_name = fields.iter().map(|field| &field.ident);
    // 😵‍💫
    let field_name_2 = field_name.clone();
    let field_type = fields.iter().map(|field| &field.ty);

    quote! {
        use async_trait::async_trait;
        
        #[async_trait]
        impl crate::utilities::parse_command::ParseCommand for #struct_name {
            async fn parse_command(command: &str) -> Result<Self, crate::utilities::parse_command::ParseError> {
                let arguments = command.chars();

                #(
                    let (#field_name, arguments) = #field_type::parse_argument(arguments).await?;
                )*

                arguments.into_iter().next().map_or(Ok(()), |_| Err(crate::utilities::parse_command::ParseError::TooManyArguments))?;

                Ok(Self { #(#field_name_2),* })
            }
        }
    }
    .into()
}
