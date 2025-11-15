use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
  DeriveInput, Ident, LitInt, Token,
  parse::{Parse, ParseStream, Result},
  parse_macro_input,
  punctuated::Punctuated,
};

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

  if types.len() == 1 {
    return TokenStream::new();
  }

  let mut impl_types = Vec::new();
  let mut bundle_types = Vec::new();
  let mut archetype_adds = Vec::new();
  let mut archetype_ids = Vec::new();
  let mut bundle_deconstruction = Vec::new();
  let mut id = Vec::new();
  for (i, t) in types.iter().enumerate() {
    impl_types.push(quote! { #t: Spawn });
    bundle_types.push(quote! { #t });
    archetype_ids.push(quote! { std::any::TypeId::of::<#t>() });
    id.push(quote! { ret = ret.merge_with(#t::archetype_id()?)?; });
    let i = syn::Index::from(i);
    let dec_name = format_ident!("value_{}", i);
    bundle_deconstruction.push(quote! { #dec_name });
    archetype_adds.push(quote! { self.#i.spawn(archetype)?; });
  }

  let output = quote! {
      impl<#(#impl_types),*> Spawn for (#(#bundle_types),*) {
          fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
              let mut ret = ArchetypeId::new(Vec::new())?;
              #(#id)*
              Ok(ret)
          }
          fn spawn(self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
              // let (#(#bundle_deconstruction),*) = self;
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

  if types.len() == 1 {
    return TokenStream::new();
  }

  let mut impl_types = Vec::new();
  let mut bundle_types = Vec::new();
  let mut item_types = Vec::new();
  let mut borrow = Vec::new();
  let mut release = Vec::new();
  let mut get = Vec::new();
  let mut id = Vec::new();
  let mut archetype_adds = Vec::new();
  let mut archetype_ids = Vec::new();
  for (i, t) in types.iter().enumerate() {
    impl_types.push(quote! { #t: Fetch<'a> });
    bundle_types.push(quote! { #t });
    item_types.push(quote! { #t::Item<'b> });
    borrow.push(quote! { #t::borrow(archetype)? });
    release.push(quote! { #t::release(archetype)? });
    archetype_ids.push(quote! { std::any::TypeId::of::<#t>() });
    id.push(quote! { ret = ret.merge_with(#t::archetype_id()?)?; });
    let i = syn::Index::from(i);
    archetype_adds.push(quote! { archetype.add(self.#i.clone())?; });
    get.push(quote! { self.#i.get(row) });
  }

  let output = quote! {
      impl<'a, #(#impl_types),*> Fetch<'a> for (#(#bundle_types),*) {
          type Item<'b> = (#(#item_types),*) where 'a: 'b, Self: 'b;

          fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
              let mut ret = ArchetypeId::new(Vec::new())?;
              #(#id)*
              Ok(ret)
          }

          fn borrow(archetype: &'a Archetype) -> Result<Self, ArchetypeError> {
              Ok((#(#borrow),*))
          }

          fn release(archetype: &'a Archetype) -> Result<(), ArchetypeError> {
              (#(#release),*);
              Ok(())
          }

          fn get<'b>(&'b mut self, row: usize) -> Self::Item<'b> {
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

#[proc_macro_derive(Component)]
pub fn derive_component(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let ident = input.ident;

  let expanded = quote! {
      impl Component for #ident {}
  };

  TokenStream::from(expanded)
}
