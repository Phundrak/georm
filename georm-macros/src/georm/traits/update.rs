use crate::georm::GeormField;
use quote::quote;

pub fn generate_update_query(table_name: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
    let update_fields: Vec<&GeormField> = fields
        .iter()
        .filter(|field| !field.is_id && !field.exclude_from_update())
        .collect();
    let update_idents: Vec<syn::Ident> = update_fields
        .iter()
        .map(|field| field.ident.clone())
        .collect();
    let id_fields: Vec<&GeormField> = fields.iter().filter(|field| field.is_id).collect();
    let id_idents: Vec<syn::Ident> = id_fields.iter().map(|f| f.ident.clone()).collect();
    let set_clauses: Vec<String> = update_fields
        .iter()
        .enumerate()
        .map(|(i, field)| format!("{} = ${}", field.ident, i + 1))
        .collect();
    let where_clauses: Vec<String> = id_fields
        .iter()
        .enumerate()
        .map(|(i, field)| format!("{} = ${}", field.ident, update_fields.len() + i + 1))
        .collect();
    let query = format!(
        "UPDATE {table_name} SET {} WHERE {} RETURNING *",
        set_clauses.join(", "),
        where_clauses.join(" AND ")
    );
    quote! {
        async fn update(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Self> {
            ::sqlx::query_as!(
                Self,
                #query,
                #(self.#update_idents),*,
                #(self.#id_idents),*
            )
            .fetch_one(pool)
            .await
        }
    }
}
