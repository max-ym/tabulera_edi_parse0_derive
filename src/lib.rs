extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use syn::export::{TokenStream, ToTokens, TokenStreamExt};
use syn::parse_macro_input::parse;
use syn::{ItemStruct};
use quote::__private::ext::RepToTokensExt;

#[proc_macro_derive(ParseError, attributes(Error))]
pub fn derive_parse_error(tokens: TokenStream) -> TokenStream {
    let struc = parse::<ItemStruct>(tokens).unwrap();
    let error_ty = {
        let mut name = None;
        for attr in struc.attrs {
            if attr.path.to_token_stream().to_string() == "Error" {
                let mut tokens = attr.tokens;
                let eq = tokens.next().unwrap();
                if eq.to_string() != "=" {
                    panic!();
                }
                name = Some(eq.to_string());
                break;
            }
        }
        name.unwrap()
    };
    let mut fields = Vec::with_capacity(struc.fields.len());
    struct MyField {
        name: String,
        ty: String,
    }
    for field in struc.fields {
        fields.push(MyField {
            ty: field.ty.to_token_stream().to_string(),
            name: field.ident.unwrap().to_string(),
        });
    }
    let mut new_fields = Vec::with_capacity(fields.len());
    for field in fields {
        let name = field.name;
        let ty = field.ty;
        new_fields.push({
                quote!(#name: std::result::Result<#ty, #error_ty>,)
        });
    }
    let mut for_brace_tokens = quote!();
    for field in new_fields {
        for_brace_tokens.append_all(field);
    }
    let result = quote!(
        #[derive(Debug)]
        pub struct ParseResult<'a> { #for_brace_tokens }
    );
    result.into()
}

