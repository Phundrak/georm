use super::composite_keys::IdType;
use super::ir::GeormField;
use quote::quote;

fn generate_find_all_query(table: &str) -> proc_macro2::TokenStream {
    let find_string = format!("SELECT * FROM {table}");
    quote! {
        async fn find_all(pool: &::sqlx::PgPool) -> ::sqlx::Result<Vec<Self>> {
            ::sqlx::query_as!(Self, #find_string).fetch_all(pool).await
        }
    }
}

fn generate_find_query(table: &str, id: &IdType) -> proc_macro2::TokenStream {
    match id {
        IdType::Simple {
            field_name,
            field_type,
        } => {
            let find_string = format!("SELECT * FROM {table} WHERE {} = $1", field_name);
            quote! {
                async fn find(pool: &::sqlx::PgPool, id: &#field_type) -> ::sqlx::Result<Option<Self>> {
                    ::sqlx::query_as!(Self, #find_string, id)
                    .fetch_optional(pool)
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
                async fn find(pool: &::sqlx::PgPool, id: &#field_type) -> ::sqlx::Result<Option<Self>> {
                    ::sqlx::query_as!(Self, #find_string, #(id.#id_members),*)
                    .fetch_optional(pool)
                    .await
                }
            }
        }
    }
}

fn generate_create_query(table: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
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

fn generate_update_query(
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

fn generate_delete_query(table: &str, id: &IdType) -> proc_macro2::TokenStream {
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

fn generate_upsert_query(
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

fn generate_get_id(id: &IdType) -> proc_macro2::TokenStream {
    match id {
        IdType::Simple {
            field_name,
            field_type,
        } => {
            quote! {
                fn get_id(&self) -> #field_type {
                    self.#field_name.clone()
                }
            }
        }
        IdType::Composite { fields, field_type } => {
            let field_names: Vec<syn::Ident> = fields.iter().map(|f| f.name.clone()).collect();
            quote! {
                fn get_id(&self) -> #field_type {
                    #field_type {
                        #(#field_names: self.#field_names),*
                    }
                }
            }
        }
    }
}

pub fn derive_trait(
    ast: &syn::DeriveInput,
    table: &str,
    fields: &[GeormField],
    id: &IdType,
) -> proc_macro2::TokenStream {
    let ty = match id {
        IdType::Simple { field_type, .. } => quote! {#field_type},
        IdType::Composite { field_type, .. } => quote! {#field_type},
    };

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // generate
    let get_all = generate_find_all_query(table);
    let get_id = generate_get_id(id);
    let find_query = generate_find_query(table, id);
    let create_query = generate_create_query(table, fields);
    let update_query = generate_update_query(table, fields, id);
    let upsert_query = generate_upsert_query(table, fields, id);
    let delete_query = generate_delete_query(table, id);
    quote! {
        impl #impl_generics Georm<#ty> for #ident #type_generics #where_clause {
            #get_all
            #get_id
            #find_query
            #create_query
            #update_query
            #upsert_query
            #delete_query
        }
    }
}
