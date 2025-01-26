use std::str::FromStr;

use crate::georm::ir::M2MRelationshipComplete;

use super::ir::GeormField;
use proc_macro2::TokenStream;
use quote::quote;

fn join_token_streams(token_streams: &[TokenStream]) -> TokenStream {
    let newline = TokenStream::from_str("\n").unwrap();
    token_streams
        .iter()
        .cloned()
        .flat_map(|ts| std::iter::once(ts).chain(std::iter::once(newline.clone())))
        .collect()
}

fn derive<T, P>(relationships: &[T], condition: P) -> TokenStream
where
    for<'a> &'a T: Into<TokenStream>,
    P: FnMut(&&T) -> bool,
{
    let implementations: Vec<TokenStream> = relationships
        .iter()
        .filter(condition)
        .map(std::convert::Into::into)
        .collect();
    join_token_streams(&implementations)
}

pub fn derive_relationships(
    ast: &syn::DeriveInput,
    struct_attrs: &super::ir::GeormStructAttributes,
    fields: &[GeormField],
    id: &GeormField,
) -> TokenStream {
    let struct_name = &ast.ident;
    let one_to_one = derive(fields, |field| field.relation.is_none());
    let one_to_many = derive(&struct_attrs.one_to_many, |_| true);
    let many_to_many: Vec<M2MRelationshipComplete> = struct_attrs
        .many_to_many
        .iter()
        .map(|v| M2MRelationshipComplete::new(v, &struct_attrs.table, id.to_string()))
        .collect();
    let many_to_many = derive(&many_to_many, |_| true);

    quote! {
        impl #struct_name {
            #one_to_one
            #one_to_many
            #many_to_many
        }
    }
}
