use crate::georm::GeormField;
use quote::quote;

pub fn generate_create_query(table: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
    let inputs: Vec<String> = (1..=fields.len()).map(|num| format!("${num}")).collect();
    let create_string = format!(
        "INSERT INTO {table} ({}) VALUES ({}) RETURNING *",
        fields
            .iter()
            .map(|f| f.ident.to_string())
            .collect::<Vec<String>>()
            .join(", "),
        inputs.join(", ")
    );
    let field_idents: Vec<syn::Ident> = fields.iter().map(|f| f.ident.clone()).collect();
    quote! {
        async fn create(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Self> {
            ::sqlx::query_as!(
                Self,
                #create_string,
                #(self.#field_idents),*
            )
            .fetch_one(pool)
            .await
        }
    }
}
