use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, Token, parse_macro_input};

struct ImplSpawnInput {
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for ImplSpawnInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(ImplSpawnInput { types })
    }
}

#[proc_macro]
pub fn impl_spawn(input: TokenStream) -> TokenStream {
    let ImplSpawnInput { types } = parse_macro_input!(input as ImplSpawnInput);

    let mut impl_types = Vec::new();
    let mut bundle_types = Vec::new();
    let mut archetype_adds = Vec::new();
    let mut archetype_ids = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Component });
        bundle_types.push(quote! { #t });
        archetype_ids.push(quote! { std::any::TypeId::of::<#t>() });
        let i = syn::Index::from(i);
        archetype_adds.push(quote! { archetype.add(self.#i.clone())?; });
    }

    if types.len() == 1 {
        bundle_types.push(quote! {});
    }

    let output = quote! {
        impl<#(#impl_types),*> Spawn for (#(#bundle_types),*) {
            fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
                ArchetypeId::new(vec![#(#archetype_ids),*])
            }

            fn spawn(self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
                #(#archetype_adds)*
                Ok(())
            }
        }
    };

    TokenStream::from(output)
}

struct ImplFetchInput {
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for ImplFetchInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(ImplFetchInput { types })
    }
}

#[proc_macro]
pub fn impl_fetch(input: TokenStream) -> TokenStream {
    let ImplFetchInput { types } = parse_macro_input!(input as ImplFetchInput);

    let mut impl_types = Vec::new();
    let mut bundle_types = Vec::new();
    let mut item_types = Vec::new();
    let mut borrow = Vec::new();
    let mut get = Vec::new();
    let mut archetype_adds = Vec::new();
    let mut archetype_ids = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Fetch<'a> });
        bundle_types.push(quote! { #t });
        item_types.push(quote! { #t::Item });
        borrow.push(quote! { #t::borrow(archetype) });
        archetype_ids.push(quote! { std::any::TypeId::of::<#t>() });
        let i = syn::Index::from(i);
        archetype_adds.push(quote! { archetype.add(self.#i.clone())?; });
        get.push(quote! { self.#i.get(row) });
    }

    if types.len() == 1 {
        bundle_types.push(quote! {});
        item_types.push(quote! {});
        borrow.push(quote! {});
        get.push(quote! {});
    }

    let output = quote! {
        impl<'a, #(#impl_types),*> Fetch<'a> for (#(#bundle_types),*) {
            type Item = (#(#item_types),*);

            fn borrow(archetype: &'a Archetype) -> Self {
                (#(#borrow),*)
            }

            fn get(&mut self, row: usize) -> Self::Item {
                (#(#get),*)
            }

            fn count(&self) -> usize {
                self.0.count()
            }
        }
    };

    TokenStream::from(output)
}

struct MultipleTuplesInput {
    mac: Ident,
    size: LitInt,
}

impl Parse for MultipleTuplesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mac = Ident::parse(input)?;
        input.parse::<Token![,]>()?;
        let size = LitInt::parse(input)?;
        Ok(MultipleTuplesInput { mac, size })
    }
}

#[proc_macro]
pub fn multiple_tuples(input: TokenStream) -> TokenStream {
    let MultipleTuplesInput { mac, size } = parse_macro_input!(input as MultipleTuplesInput);
    let size = size.base10_parse::<u8>().unwrap();

    let mut names = Vec::new();
    let mut output = Vec::new();
    for current_size in 1..size {
        let name = format_ident!("T{}", current_size);
        names.push(quote! { #name });
        output.push(quote! { #mac!(#(#names),*); });
    }

    TokenStream::from(quote! { #(#output)* })
}
