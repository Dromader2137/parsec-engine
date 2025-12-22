use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::{ToTokens, format_ident, quote};
use syn::{
    DeriveInput, FnArg, Ident, ItemFn, LitInt, Pat, PatType, Token,
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    punctuated::Punctuated,
};

struct ImplSpawnInput {
    types: Punctuated<Ident, Token![,]>,
}

impl Parse for ImplSpawnInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let types: Punctuated<Ident, Token![,]> =
            Punctuated::parse_terminated(input)?;
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
        let types: Punctuated<Ident, Token![,]> =
            Punctuated::parse_terminated(input)?;
        Ok(ImplFetchInput { types })
    }
}

#[proc_macro]
pub fn impl_fetch(input: TokenStream) -> TokenStream {
    let ImplFetchInput { types } = parse_macro_input!(input as ImplFetchInput);

    if types.len() == 1 {
        return TokenStream::new();
    }

    let t_one = types.get(0).unwrap();
    let mut impl_types = Vec::new();
    let mut bundle_types = Vec::new();
    let mut state_types = Vec::new();
    let mut item_types = Vec::new();
    let mut prepare = Vec::new();
    let mut release = Vec::new();
    let mut get = Vec::new();
    let mut id = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Fetch });
        bundle_types.push(quote! { #t });
        item_types.push(quote! { #t::Item<'a> });
        state_types.push(quote! { #t::State });
        prepare.push(quote! { #t::prepare(archetype)? });
        id.push(quote! { ret = ret.merge_with(#t::archetype_id()?)?; });
        let i = syn::Index::from(i);
        release.push(quote! { #t::release(state.#i)? });
        get.push(quote! { #t::get(state.#i.clone(), row) });
    }

    let output = quote! {
        impl<#(#impl_types),*> Fetch for (#(#bundle_types),*) {
            type Item<'a> = (#(#item_types),*) where Self: 'a;
            type State = (#(#state_types),*);

            fn archetype_id() -> Result<ArchetypeId, ArchetypeError> {
                let mut ret = ArchetypeId::new(Vec::new())?;
                #(#id)*
                Ok(ret)
            }

            fn prepare(archetype: &Archetype) -> Result<Self::State, ArchetypeError> {
                Ok((#(#prepare),*))
            }

            fn release(state: Self::State) -> Result<(), ArchetypeError> {
                (#(#release),*);
                Ok(())
            }

            fn get<'a>(state: Self::State, row: usize) -> Self::Item<'a> {
                (#(#get),*)
            }

            fn len(state: &Self::State) -> usize {
                #t_one::len(&state.0)
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
    let MultipleTuplesInput { mac, size } =
        parse_macro_input!(input as MultipleTuplesInput);
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
        impl Copy for #ident {}
        impl Clone for #ident {
            fn clone(&self) -> Self {
                *self
            }
        }
        impl Component for #ident {}
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn system(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let struct_name =
        format_ident!("{}", fn_name.to_string().to_case(Case::Pascal));
    let struct_name_str = struct_name.to_token_stream().to_string();

    let found_crate = crate_name("parsec-engine")
        .expect("parsec-engine is present in `Cargo.toml`");

    let engine_crate = match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( ::#ident )
        },
    };

    let borrows = input_fn.sig.inputs.iter().map(|arg| match arg {
        FnArg::Typed(PatType { pat, ty, .. }) => {
            let argument_type = ty;

            let (argument_name, mutability) = match &**pat {
                Pat::Ident(pat_ident) => {
                    let is_mut = pat_ident.mutability.is_some();
                    let ident = &pat_ident.ident;
                    // is_mut == true for `mut b`
                    (ident, is_mut)
                },
                _ => panic!("Only ident is supported inside systems"),
            };

            if mutability {
                quote! {
                    let mut #argument_name = <#argument_type as #engine_crate::ecs::system::SystemInput>::borrow();
                }
            } else {
                quote! {
                    let #argument_name = <#argument_type as #engine_crate::ecs::system::SystemInput>::borrow();
                }
            }
        },
        FnArg::Receiver(_) => {
            panic!("Systems cannot take &self or &mut self");
        },
    });

    let argument_names = input_fn.sig.inputs.iter().map(|arg| match arg {
        FnArg::Typed(PatType { pat, .. }) => {
            let argument_name = match &**pat {
                Pat::Ident(pat_ident) => {
                    let ident = &pat_ident.ident;
                    ident
                },
                _ => panic!("Only ident is supported inside systems"),
            };
            quote! { #argument_name }
        },
        _ => unreachable!(),
    });

    let output = quote! {
        #input_fn

        pub struct #struct_name;

        impl #struct_name {
            pub fn new() -> Box<Self> {
                Box::new(Self)
            }
        }

        impl #engine_crate::ecs::system::System for #struct_name {
            fn name(&self) -> &'static str {
                #struct_name_str
            }

            fn run(&mut self) {
                #(#borrows)*
                #fn_name( #(#argument_names),* );
            }
        }
    };

    output.into()
}
