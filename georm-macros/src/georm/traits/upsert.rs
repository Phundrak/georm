use crate::georm::{GeormField, IdType, ir::GeneratedType};
use quote::quote;

pub fn generate_upsert_query(
    table: &str,
    fields: &[GeormField],
    id: &IdType,
) -> proc_macro2::TokenStream {
    let fields: Vec<&GeormField> = fields
        .iter()
        .filter(|field| !matches!(field.generated_type, GeneratedType::Always))
        .collect();
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
        .filter(|f| !f.is_id)
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
        async fn upsert<'e, E>(&self, mut executor: E) -> ::sqlx::Result<Self>
        where
            E: ::sqlx::Executor<'e, Database = ::sqlx::Postgres>
        {
            ::sqlx::query_as!(
                Self,
                #upsert_string,
                #(self.#field_idents),*
            )
            .fetch_one(executor)
            .await
        }
    }
}
