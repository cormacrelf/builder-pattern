use super::attributes::FieldAttributes;

use core::cmp::Ordering;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::{Attribute, Token, Type, Visibility};

pub struct Field {
    pub vis: Visibility,
    pub ident: Ident,
    pub ty: Type,
    pub attrs: FieldAttributes,
}

impl Field {
    pub fn documents(&self) -> Vec<Attribute> {
        self.attrs
            .documents
            .iter()
            .filter(|a| a.path.is_ident("doc"))
            .map(|a| a.to_owned())
            .collect()
    }

    pub fn type_documents(&self) -> String {
        let ty_token = self.ty.clone().into_token_stream();
        if self.attrs.use_into {
            format!("Into<{}>", ty_token)
        } else {
            ty_token.to_string()
        }
    }

    pub fn tokenize_replacement_params(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        if self.attrs.replace_generics.is_empty() {
            return stream;
        }
        let underscored = self.attrs.replace_generics.iter().map(|ident| {
            let string = ident.to_string() + "_";
            Ident::new(&string, ident.span())
        });
        <Token![<]>::default().to_tokens(&mut stream);
        stream.append_terminated(underscored, <Token![,]>::default());
        <Token![>]>::default().to_tokens(&mut stream);
        stream
    }
}

impl Ord for Field {
    fn cmp(&self, other: &Field) -> Ordering {
        self.ident.cmp(&other.ident)
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Field) -> Option<Ordering> {
        Some(self.ident.cmp(&other.ident))
    }
}

impl Eq for Field {}

impl PartialEq for Field {
    fn eq(&self, other: &Field) -> bool {
        self.ident.eq(&other.ident)
    }
}
