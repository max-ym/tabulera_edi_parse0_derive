extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
#[macro_use]
extern crate quote;

use syn::export::{TokenStream, ToTokens, TokenStreamExt, TokenStream2};
use syn::parse_macro_input::parse;
use syn::{ItemStruct, ExprParen, TypePath};

struct MyField {
    name: proc_macro2::TokenStream,
    ty: proc_macro2::TokenStream,
}

#[proc_macro_derive(ParseError, attributes(ParseDestination))]
pub fn derive_parse_error(tokens: TokenStream) -> TokenStream {
    let struc = parse::<ItemStruct>(tokens).expect("Expected struct");
    let mut result = quote!();
    let mut fields = Vec::with_capacity(struc.fields.len());
    let dest = {
        let mut dest = None;
        for attr in struc.attrs {
            if attr.path.to_token_stream().to_string() == "ParseDestination" {
                let tokens = attr.tokens;
                dest = Some({
                    let mut q = quote!();
                    let val = parse::<ExprParen>(tokens.into()).unwrap();
                    q.append_all(val.expr.into_token_stream());
                    q
                });
                break;
            }
        }
        dest.unwrap()
    };
    for field in struc.fields {
        let ts = field.ty.into_token_stream();
        let v = parse::<TypePath>(ts.into()).unwrap();
        let ty = if let syn::PathArguments::AngleBracketed(v)
                = &v.path.segments.first().unwrap().arguments {
            v.args.first().unwrap().clone().into_token_stream()
        } else {
            unreachable!()
        };
        fields.push(MyField {
            ty,
            name: field.ident.expect("Expected identifier").to_token_stream(),
        });
    }
    let mut new_fields = Vec::with_capacity(fields.len());
    for field in &fields {
        let name = field.name.to_owned();
        let ty = field.ty.to_owned();
        new_fields.push({
                quote!(pub #name: #ty,)
        });
    }
    let mut for_brace_tokens = quote!();
    for field in new_fields {
        for_brace_tokens.append_all(field);
    }
    for_brace_tokens.append_all(quote!(
        _phantom: std::marker::PhantomData<&'a ()>,
    ));
    result.append_all(quote!(
        #[derive(Debug, Serialize)]
        pub struct #dest<'a> { #for_brace_tokens }
    ).into_iter());
    result.append_all(validate(&fields, dest).into_iter());
    result.into()
}

fn validate(fields: &Vec<MyField>, dest: TokenStream2) -> proc_macro2::TokenStream {
    let if_err = {
        let mut tokens = quote!();
        for field in fields {
            let name = &field.name;
            tokens.append_all(quote!(
                s.#name.is_err() ||
            ));
        }
        tokens.append_all(quote!(false {
            Err(ParseError::DataError(self))
        }));
        tokens
    };
    let else_unwrap_brace = {
        let ok_brace = {
            let mut tokens = quote!();
            for field in fields {
                let name = &field.name;
                tokens.append_all(quote!(
                    #name: self.#name.unwrap(),
                ));
            }
            tokens.append_all(quote!(
                _phantom: std::marker::PhantomData,
            ));
            tokens
        };
        let tokens = quote!(Ok(
            #dest {
                #ok_brace
            }
        ));
        tokens
    };
    quote!(
        impl<'a> ParseResult<'a> {
            pub fn validate(self) -> Output<'a> {
                let s = &self;
                if #if_err else {
                    #else_unwrap_brace
                }
            }
        }
    )
}
