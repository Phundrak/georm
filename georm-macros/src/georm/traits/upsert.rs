use crate::georm::{GeormField, IdType};
use quote::quote;

pub fn generate_upsert_query(
    table: &str,
    fields: &[GeormField],
    id: &IdType,
) -> proc_macro2::TokenStream {
    let inputs: Vec<String> = (1..=fields.len()).map(|num| format!("${num}")).collect();
    let columns = fields
        .iter()
        .map(|f| f.ident.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    let primary_key: proc_macro2::TokenStream = match id {
        IdType::Simple { field_name, .. } => quote! {#field_name},
        IdType::Composite { fields, .. } => {
            let field_names: Vec<syn::Ident> = fields.iter().map(|f| f.name.clone()).collect();
            quote! {
                #(#field_names),*
            }
        }
    };

    // For ON CONFLICT DO UPDATE, exclude the ID field from updates
    let update_assignments = fields
        .iter()
        .filter(|f| !f.id)
        .map(|f| format!("{} = EXCLUDED.{}", f.ident, f.ident))
        .collect::<Vec<String>>()
        .join(", ");

    let upsert_string = format!(
        "INSERT INTO {table} ({columns}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {update_assignments} RETURNING *",
        inputs.join(", "),
        primary_key
    );

    let field_idents: Vec<syn::Ident> = fields.iter().map(|f| f.ident.clone()).collect();

    quote! {
        async fn create_or_update(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Self> {
            ::sqlx::query_as!(
                Self,
                #upsert_string,
                #(self.#field_idents),*
            )
            .fetch_one(pool)
            .await
        }
    }
}
