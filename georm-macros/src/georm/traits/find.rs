use crate::georm::IdType;
use quote::quote;

pub fn generate_find_all_query(table: &str) -> proc_macro2::TokenStream {
    let find_string = format!("SELECT * FROM {table}");
    quote! {
        async fn find_all<'e, E>(mut executor: E) -> ::sqlx::Result<Vec<Self>>
        where
            E: ::sqlx::Executor<'e, Database = ::sqlx::Postgres>
        {
            ::sqlx::query_as!(Self, #find_string).fetch_all(executor).await
        }
    }
}

pub fn generate_find_query(table: &str, id: &IdType) -> proc_macro2::TokenStream {
    match id {
        IdType::Simple {
            field_name,
            field_type,
        } => {
            let find_string = format!("SELECT * FROM {table} WHERE {} = $1", field_name);
            quote! {
                async fn find<'e, E>(mut executor: E, id: &#field_type) -> ::sqlx::Result<Option<Self>>
                where
                    E: ::sqlx::Executor<'e, Database = ::sqlx::Postgres>
                {
                    ::sqlx::query_as!(Self, #find_string, id)
                        .fetch_optional(executor)
                        .await
                }
            }
        }
        IdType::Composite { fields, field_type } => {
            let id_match_string = fields
                .iter()
                .enumerate()
                .map(|(i, field)| format!("{} = ${}", field.name, i + 1))
                .collect::<Vec<String>>()
                .join(" AND ");
            let id_members: Vec<syn::Ident> =
                fields.iter().map(|field| field.name.clone()).collect();
            let find_string = format!("SELECT * FROM {table} WHERE {id_match_string}");
            quote! {
                async fn find<'e, E>(mut executor: E, id: &#field_type) -> ::sqlx::Result<Option<Self>>
                where
                    E: ::sqlx::Executor<'e, Database = ::sqlx::Postgres>
                {
                    ::sqlx::query_as!(Self, #find_string, #(id.#id_members),*)
                        .fetch_optional(executor)
                        .await
                }
            }
        }
    }
}
