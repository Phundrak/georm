use crate::georm::IdType;
use quote::quote;

pub fn generate_delete_query(table: &str, id: &IdType) -> proc_macro2::TokenStream {
    let where_clause = match id {
        IdType::Simple { field_name, .. } => format!("{} = $1", field_name),
        IdType::Composite { fields, .. } => fields
            .iter()
            .enumerate()
            .map(|(i, field)| format!("{} = ${}", field.name, i + 1))
            .collect::<Vec<String>>()
            .join(" AND "),
    };
    let query_args = match id {
        IdType::Simple { .. } => quote! { id },
        IdType::Composite { fields, .. } => {
            let fields: Vec<syn::Ident> = fields.iter().map(|f| f.name.clone()).collect();
            quote! { #(id.#fields), * }
        }
    };
    let id_type = match id {
        IdType::Simple { field_type, .. } => quote! { #field_type },
        IdType::Composite { field_type, .. } => quote! { #field_type },
    };
    let delete_string = format!("DELETE FROM {table} WHERE {where_clause}");
    quote! {
        async fn delete_by_id(pool: &::sqlx::PgPool, id: &#id_type) -> ::sqlx::Result<u64> {
            let rows_affected = ::sqlx::query!(#delete_string, #query_args)
                .execute(pool)
                .await?
                .rows_affected();
            Ok(rows_affected)
        }

        async fn delete(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<u64> {
            Self::delete_by_id(pool, &self.get_id()).await
        }
    }
}
