use std::str::FromStr;

use crate::georm::ir::m2m_relationship::M2MRelationshipComplete;

use super::composite_keys::IdType;
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

fn derive<T>(relationships: &[T]) -> TokenStream
where
    for<'a> &'a T: Into<TokenStream>,
{
    let implementations: Vec<TokenStream> =
        relationships.iter().map(std::convert::Into::into).collect();
    join_token_streams(&implementations)
}

pub fn derive_relationships(
    ast: &syn::DeriveInput,
    struct_attrs: &super::ir::GeormStructAttributes,
    fields: &[GeormField],
    id: &IdType,
) -> TokenStream {
    let id = match id {
        IdType::Simple {
            field_name,
            field_type: _,
        } => field_name.to_string(),
        IdType::Composite {
            fields: _,
            field_type: _,
        } => {
            eprintln!(
                "Warning: entity {}: Relationships are not supported for entities with composite primary keys yet",
                ast.ident
            );
            return quote! {};
        }
    };
    let struct_name = &ast.ident;
    let one_to_one_local = derive(fields);
    let one_to_one_remote = derive(&struct_attrs.one_to_one);
    let one_to_many = derive(&struct_attrs.one_to_many);
    let many_to_many: Vec<M2MRelationshipComplete> = struct_attrs
        .many_to_many
        .iter()
        .map(|v| M2MRelationshipComplete::new(v, &struct_attrs.table, &id))
        .collect();
    let many_to_many = derive(&many_to_many);

    quote! {
        impl #struct_name {
            #one_to_one_local
            #one_to_one_remote
            #one_to_many
            #many_to_many
        }
    }
}
