use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, LitInt, Token};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result};

struct ExtractTupleInput {
    tuple_var: Expr,
    indices: Punctuated<LitInt, Token![,]>,
}

impl Parse for ExtractTupleInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let tuple_var: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let indices: Punctuated<LitInt, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(ExtractTupleInput { tuple_var, indices })
    }
}

#[proc_macro]
pub fn extract_tuple(input: TokenStream) -> TokenStream {
    let ExtractTupleInput { tuple_var, indices } = parse_macro_input!(input as ExtractTupleInput);
    
    let mut extracted_elements = Vec::new();
    
    for index in indices.iter() {
        let idx: usize = index.base10_parse().unwrap();
        let idx_final = syn::Index::from(idx);
        extracted_elements.push(quote! { #tuple_var.#idx_final })
    }

    let expanded = quote! {
        (#(#extracted_elements),*)
    };

    TokenStream::from(expanded)
}
