use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    DeriveInput, Ident, LitInt, LitStr, Token,
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
        let i = syn::Index::from(i);
        let dec_name = format_ident!("value_{}", i);
        id.push(quote! { ret = ret.merge_with(self.#i.archetype_id()?)?; });
        bundle_deconstruction.push(quote! { #dec_name });
        archetype_adds.push(quote! { self.#i.spawn(archetype)?; });
    }

    let output = quote! {
        impl<#(#impl_types),*> Spawn for (#(#bundle_types),*) {
            fn archetype_id(&self) -> Result<ArchetypeId, ArchetypeError> {
                let mut ret = ArchetypeId::new(Vec::new())?;
                #(#id)*
                Ok(ret)
            }
            fn spawn(&self, archetype: &mut Archetype) -> Result<(), ArchetypeError> {
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

struct EmbeddShaderInput {
    shader_type: LitStr,
    shader_code: LitStr,
}

impl Parse for EmbeddShaderInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let shader_type = input.parse()?;
        input.parse::<Token![,]>()?;
        let shader_code = input.parse()?;
        Ok(EmbeddShaderInput { shader_type, shader_code })
    }
}

fn reinterpret_shader_code(bytes: &[u8]) -> Vec<u32> {
    let size = bytes.len();
    let words = size / 4;
    let mut result =
        unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast::<u32>(), words) }.to_vec();
    const MAGIC_NUMBER: u32 = 0x0723_0203;
    if !result.is_empty() && result[0] == MAGIC_NUMBER.swap_bytes() {
        for word in result.iter_mut() {
            *word = word.swap_bytes();
        }
    }
    result
}

#[proc_macro]
pub fn embed_shader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as EmbeddShaderInput);

    let compiler = shaderc::Compiler::new().unwrap();
    let spv = compiler
        .compile_into_spirv(
            &input.shader_code.value(),
            match input.shader_type.value().as_str() {
                "vert" => shaderc::ShaderKind::DefaultVertex,
                "frag" => shaderc::ShaderKind::DefaultFragment,
                _ => panic!("invalid shader extension"),
            },
            "ast",
            "main",
            None,
        )
        .unwrap();
    let code = reinterpret_shader_code(spv.as_binary_u8());
    TokenStream::from(quote! { &[#(#code),*] })
}
