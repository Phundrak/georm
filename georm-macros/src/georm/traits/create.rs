use crate::georm::GeormField;
use quote::quote;

pub fn generate_create_query(table_name: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
    let insert_fields: Vec<&GeormField> = fields
        .iter()
        .filter(|field| !field.exclude_from_insert())
        .collect();
    let field_names: Vec<String> = insert_fields
        .iter()
        .map(|field| field.ident.to_string())
        .collect();
    let field_idents: Vec<syn::Ident> = insert_fields
        .iter()
        .map(|field| field.ident.clone())
        .collect();
    let placeholders: Vec<String> = (1..=insert_fields.len()).map(|i| format!("${i}")).collect();
    let query = format!(
        "INSERT INTO {table_name} ({}) VALUES ({}) RETURNING *",
        field_names.join(", "),
        placeholders.join(", ")
    );
    quote! {
        async fn create<'e, E>(&self, mut executor: E) -> ::sqlx::Result<Self>
        where
            E: ::sqlx::Executor<'e, Database = ::sqlx::Postgres>
        {
            ::sqlx::query_as!(
                Self,
                #query,
                #(self.#field_idents),*
            )
            .fetch_one(executor)
            .await
        }
    }
}
