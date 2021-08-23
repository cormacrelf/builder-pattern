use crate::struct_input::StructInput;

use proc_macro2::TokenStream;
use quote::ToTokens;
use std::str::FromStr;
use syn::{parse_quote, spanned::Spanned, Attribute};

/// Implementation for the given structure.
/// It creates a `new` function.
pub struct StructImpl<'a> {
    pub input: &'a StructInput,
}

impl<'a> ToTokens for StructImpl<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.input.ident;
        let vis = &self.input.vis;
        let where_clause = &self.input.generics.where_clause;
        let builder_name = self.input.builder_name();

        let lifetimes = self.input.lifetimes();
        let impl_tokens = self.input.tokenize_impl();
        let empty_generics = self.empty_generics();
        let ty_tokens = self.input.tokenize_types();

        let builder_init_args = self.builder_init_args();
        let docs = self.documents();

        tokens.extend(quote! {
            impl <#impl_tokens> #ident <#(#lifetimes,)* #ty_tokens> #where_clause {
                #(#docs)*
                #vis fn new() -> #builder_name<#(#lifetimes,)* #ty_tokens #(#empty_generics),*> {
                    #builder_name {
                        _phantom: ::std::marker::PhantomData,
                        #(#builder_init_args),*
                    }
                }
            }
        });
    }
}

impl<'a> StructImpl<'a> {
    pub fn new(input: &'a StructInput) -> StructImpl<'a> {
        StructImpl { input }
    }

    /// An iterator to describe initial state of builder.
    fn empty_generics(&self) -> impl Iterator<Item = TokenStream> {
        (0..(self.input.required_fields.len() + self.input.optional_fields.len()))
            .into_iter()
            .map(|_| TokenStream::from_str("()").unwrap())
    }

    /// An iterator for initialize arguments of the builder.
    /// Required fields are filled with `None`, optional fields are filled with given value via `default` attribute.
    fn builder_init_args(&'_ self) -> impl '_ + Iterator<Item = TokenStream> {
        self.input
            .required_fields
            .iter()
            .map(|f| {
                let ident = &f.ident;
                quote! {
                    #ident: None
                }
            })
            .chain(self.input.optional_fields.iter().map(|f| {
                let (ident, expr) = (&f.ident, &f.attrs.default.as_ref());
                quote_spanned! { expr.span() =>
                    #ident: Some(#expr)
                }
            }))
    }

    fn documents(&self) -> Vec<Attribute> {
        let mut docs: Vec<Attribute> = Vec::new();

        docs.push(parse_quote!(#[doc=" Creating a builder."]));

        docs.push(parse_quote!(#[doc=" ## Required Fields"]));
        for f in self.input.required_fields.iter() {
            let ident = &f.ident;
            let ty = &f.ty;
            let use_into = f.attrs.use_into;

            let ty_tokens = if use_into {
                format!("Into<{}>", ty.into_token_stream())
            } else {
                ty.into_token_stream().to_string()
            };

            let doc = format!(" ### `{}`\n - Type: {}", ident, ty_tokens);
            docs.push(parse_quote!(#[doc=#doc]));
            docs.append(f.documents().as_mut());
        }

        docs.push(parse_quote!(#[doc=" ## Optional Fields"]));
        for f in self.input.optional_fields.iter() {
            let ident = &f.ident;
            let ty = &f.ty;
            let default = f.attrs.default.as_ref().unwrap();

            let doc = format!(
                " ### `{}`\n - Type: `{}`\n - Default: `{}`",
                ident,
                ty.into_token_stream(),
                default.into_token_stream()
            );
            docs.push(parse_quote!(#[doc=#doc]));
            docs.append(f.documents().as_mut());
        }

        docs
    }
}
