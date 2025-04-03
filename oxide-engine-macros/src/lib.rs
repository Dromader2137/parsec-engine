use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, Token};
use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream, Result};

struct ImplBundleInput {
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for ImplBundleInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(ImplBundleInput { types })
    }
}

#[proc_macro]
pub fn impl_bundle(input: TokenStream) -> TokenStream {
    let ImplBundleInput { types } = parse_macro_input!(input as ImplBundleInput);

    if types.len() == 1 {
        let t = types.get(0).unwrap();
        let output = quote! {
            impl<#t: Clone + Sized + Send + Sync + 'static> Bundle for (#t, ) {
                fn type_id(&self) -> TypeId {
                    TypeId::of::<Self>()
                }

                fn add_to(&self, arch: &mut Archetype) {
                    arch.add(self.clone());
                }
            }
        };
        return TokenStream::from(output);
    }

    let mut impl_types = Vec::new();
    let mut for_types = Vec::new();
    let mut adds = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Clone + Sized + Send + Sync + 'static });
        for_types.push(quote! { #t });
        let i = syn::Index::from(i);
        adds.push(quote! { arch.add(self.#i.clone()); });
    }

    let output = quote! {
        impl<#(#impl_types),*> Bundle for (#(#for_types),*) {
            fn type_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }

            fn add_to(&self, arch: &mut Archetype) {
                #(#adds)*
            }
        }
    };

    TokenStream::from(output)
}

struct ImplFromColumnsInput {
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for ImplFromColumnsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(ImplFromColumnsInput { types })
    }
}

#[proc_macro]
pub fn impl_from_columns(input: TokenStream) -> TokenStream {
    let ImplFromColumnsInput { types } = parse_macro_input!(input as ImplFromColumnsInput);
    
    if types.len() == 1 {
        let t = types.get(0).unwrap();
        let output = quote! {
            impl<#t: Clone + Sized + Send + Sync + 'static> FromColumns for (#t, ) {
                fn extend_from_columns(vec: &mut Vec<Self>, arch: &Archetype) {
                    let sov = arch.get::<#t>().unwrap().to_vec();
                    for i in 0..arch.bundle_count {
                        vec.push((sov[i as usize].clone(), ));
                    }
                }
            }
        };
        return TokenStream::from(output);
    }

    let mut impl_types = Vec::new();
    let mut for_types = Vec::new();
    let mut gets = Vec::new();
    let mut conv = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Sized + Clone + Send + Sync + 'static });
        for_types.push(quote! { #t });
        let i = syn::Index::from(i);
        gets.push(quote! { arch.get::<#t>().unwrap() });
        conv.push(quote! { sov.#i[i as usize].clone() });
    }

    let output = quote! {
        impl<#(#impl_types),*> FromColumns for (#(#for_types),*) {
            fn extend_from_columns(vec: &mut Vec<Self>, arch: &Archetype) {
                let sov = (#(#gets),*);
                for i in 0..arch.bundle_count {
                    vec.push((#(#conv),*))
                }
            }
        }
    };

    TokenStream::from(output)
}
