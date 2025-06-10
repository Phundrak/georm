use crate::georm::{GeormField, IdType};
use quote::quote;

pub fn generate_update_query(
    table: &str,
    fields: &[GeormField],
    id: &IdType,
) -> proc_macro2::TokenStream {
    let non_id_fields: Vec<syn::Ident> = fields
        .iter()
        .filter_map(|f| if f.id { None } else { Some(f.ident.clone()) })
        .collect();
    let update_columns = non_id_fields
        .iter()
        .enumerate()
        .map(|(i, field)| format!("{} = ${}", field, i + 1))
        .collect::<Vec<String>>()
        .join(", ");
    let mut all_fields = non_id_fields.clone();
    let where_clause = match id {
        IdType::Simple { field_name, .. } => {
            let where_clause = format!("{} = ${}", field_name, non_id_fields.len() + 1);
            all_fields.push(field_name.clone());
            where_clause
        }
        IdType::Composite { fields, .. } => fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let where_clause = format!("{} = ${}", field.name, non_id_fields.len() + i + 1);
                all_fields.push(field.name.clone());
                where_clause
            })
            .collect::<Vec<String>>()
            .join(" AND "),
    };
    let update_string =
        format!("UPDATE {table} SET {update_columns} WHERE {where_clause} RETURNING *");
    quote! {
        async fn update(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Self> {
            ::sqlx::query_as!(
                Self, #update_string, #(self.#all_fields),*
            )
            .fetch_one(pool)
            .await
        }
    }
}
