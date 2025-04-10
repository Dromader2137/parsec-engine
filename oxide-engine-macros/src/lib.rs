use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, LitInt, Token, parse_macro_input};

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

                fn add_to(&self, arch: &mut Archetype) -> Result<(), ArchetypeError> {
                    arch.add(self.0.clone())?;
                    Ok(())
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
        adds.push(quote! { arch.add(self.#i.clone())?; });
    }

    let output = quote! {
        impl<#(#impl_types),*> Bundle for (#(#for_types),*) {
            fn type_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }

            fn add_to(&self, arch: &mut Archetype) -> Result<(), ArchetypeError> {
                #(#adds)*
                Ok(())
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

    let len = types.len();
    let mut impl_types = Vec::new();
    let mut bundle_types = Vec::new();
    let mut query_types = Vec::new();
    let mut gets = Vec::new();
    let mut query_ref = Vec::new();
    let mut query_fields = Vec::new();
    let mut query_field_names = Vec::new();
    let mut query_init = Vec::new();
    let mut query_get = Vec::new();
    let mut query_drop = Vec::new();
    let mut query_borrow = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Sized + Clone + Send + Sync + 'static });
        bundle_types.push(quote! { #t });
        query_types.push(quote! { #t });
        gets.push(quote! { arch.get::<#t>()? });
        query_ref.push(quote! { &'a #t });
        let field = format_ident!("i_{}", i);
        query_fields.push(quote! { #field: (std::slice::Iter<'a, #t>, &'a ColumnStateWrapper) });
        query_field_names.push(quote! { #field });
        query_get.push(quote! { self.#field.0.next()? });
        let index = syn::Index::from(i);
        query_init.push(quote! { ((*sov.#index.0).iter(), sov.#index.1) });
        query_drop.push(quote! { self.#field.1.free().unwrap(); });
        query_borrow.push(quote! { #field.1.borrow()?; });
    }

    if types.len() == 1 {
        bundle_types.push(quote! {});
        query_ref.push(quote! {});
        gets.push(quote! {});
    }

    let query_name = format_ident!("Query{}", len);

    let output = quote! {
        struct #query_name<'a, #(#bundle_types),*> {
            #(#query_fields),*
        }

        impl<'a, #(#query_types),*> #query_name<'a, #(#bundle_types),*> {
            fn new(#(#query_fields),*) -> Result<#query_name<'a, #(#bundle_types),*>, ArchetypeError> {
                #(#query_borrow)*
                Ok(#query_name {#(#query_field_names),*})
            }
        }

        impl<'a, #(#query_types),*> Drop for #query_name<'a, #(#query_types),*> {
            fn drop(&mut self) {
                #(#query_drop)*
            }
        }

        impl<'a, #(#query_types),*> Iterator for #query_name<'a, #(#query_types),*> {
            type Item = (#(#query_ref),*);
            fn next(&mut self) -> Option<Self::Item> {
                Some((
                    #(#query_get),*,
                ))
            }
        }

        impl<'a, #(#impl_types),*> FromColumns<'a> for (#(#bundle_types),*) {
            type Output = (#(#query_ref),*);
            fn iter_from_columns<'b>(arch: &'b Archetype) -> Result<impl Iterator<Item = Self::Output>, ArchetypeError> where 'b: 'a  {
                let sov = (#(#gets),*);
                unsafe {
                #query_name::new(#(#query_init),*)
                }
            }
        }
    };

    TokenStream::from(output)
}

#[proc_macro]
pub fn impl_from_columns_mut(input: TokenStream) -> TokenStream {
    let ImplFromColumnsInput { types } = parse_macro_input!(input as ImplFromColumnsInput);

    let len = types.len();
    let mut impl_types = Vec::new();
    let mut bundle_types = Vec::new();
    let mut query_types = Vec::new();
    let mut gets = Vec::new();
    let mut query_ref = Vec::new();
    let mut query_fields = Vec::new();
    let mut query_field_names = Vec::new();
    let mut query_init = Vec::new();
    let mut query_get = Vec::new();
    let mut query_drop = Vec::new();
    let mut query_borrow = Vec::new();
    for (i, t) in types.iter().enumerate() {
        impl_types.push(quote! { #t: Sized + Clone + Send + Sync + 'static });
        bundle_types.push(quote! { #t });
        query_types.push(quote! { #t });
        query_ref.push(quote! { &'a mut #t });
        gets.push(quote! { arch.get_mut::<#t>()? });
        let field = format_ident!("i_{}", i);
        query_fields.push(quote! { #field: (std::slice::IterMut<'a, #t>, &'a ColumnStateWrapper) });
        query_field_names.push(quote! { #field });
        query_get.push(quote! { self.#field.0.next()? });
        let index = syn::Index::from(i);
        query_init.push(quote! { ((*sov.#index.0).iter_mut(), sov.#index.1) });
        query_drop.push(quote! { self.#field.1.free().unwrap(); });
        query_borrow.push(quote! { #field.1.borrow_mut()?; });
    }

    if types.len() == 1 {
        bundle_types.push(quote! {});
        query_ref.push(quote! {});
        gets.push(quote! {});
    }

    let query_name = format_ident!("QueryMut{}", len);

    let output = quote! {
        struct #query_name<'a, #(#bundle_types),*> {
            #(#query_fields),*
        }

        impl<'a, #(#query_types),*> #query_name<'a, #(#bundle_types),*> {
            fn new(#(#query_fields),*) -> Result<#query_name<'a, #(#bundle_types),*>, ArchetypeError> {
                #(#query_borrow)*
                Ok(#query_name {#(#query_field_names),*})
            }
        }

        impl<'a, #(#query_types),*> Drop for #query_name<'a, #(#query_types),*> {
            fn drop(&mut self) {
                #(#query_drop)*
            }
        }

        impl<'a, #(#bundle_types),*> Iterator for #query_name<'a, #(#bundle_types),*> {
            type Item = (#(#query_ref),*);
            fn next(&mut self) -> Option<Self::Item> {
                Some((
                    #(#query_get),*,
                ))
            }
        }

        impl<'a, #(#impl_types),*> FromColumnsMut<'a> for (#(#bundle_types),*) {
            type Output = (#(#query_ref),*);
            fn iter_from_columns<'b>(arch: &'b Archetype) -> Result<impl Iterator<Item = Self::Output>, ArchetypeError> where 'b: 'a  {
                let sov = (#(#gets),*);
                unsafe {
                    #query_name::new(#(#query_init),*)
                }
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
