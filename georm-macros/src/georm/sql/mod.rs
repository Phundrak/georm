use crate::georm::composite_keys::IdType;
use crate::georm::ir::GeneratedType;
use crate::georm::ir::GeormField;
use quote::quote;

mod postgres;
mod sqlite;

#[cfg(feature = "postgres")]
pub use postgres::PostgresDialect;
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteDialect;

#[cfg(feature = "postgres")]
pub type ActiveDialect = PostgresDialect;
#[cfg(feature = "sqlite")]
pub type ActiveDialect = SqliteDialect;

#[cfg(feature = "postgres")]
pub const DIALECT: ActiveDialect = PostgresDialect;
#[cfg(feature = "sqlite")]
pub const DIALECT: ActiveDialect = SqliteDialect;

/// How a relation-lookup query should fetch its result.
pub enum FetchKind {
    /// `fetch_one`, returns `Entity`
    One,
    /// `fetch_optional`, returns `Option<Entity>`
    Optional,
    /// `fetch_all`, returns `Vec<Entity>`
    Many,
}

/// Abstracts the SQL dialect-specific parts of Georm's generated code: bind
/// parameter placeholder syntax, and the `sqlx` database/row types used in
/// generated trait bounds and `FromRow` impls. `RETURNING` and
/// `ON CONFLICT ... DO UPDATE` are supported identically by every dialect
/// Georm targets, so query shape itself does not need to vary.
pub trait SqlDialect {
    fn placeholder(&self, index: usize) -> String;
    fn database_type(&self) -> proc_macro2::TokenStream;
    fn row_type(&self) -> proc_macro2::TokenStream;

    /// Like [`SqlDialect::placeholder`], but for placeholder lists whose
    /// length is only known at runtime (the defaultable-struct insert, which
    /// varies its column count based on which `Option` fields are `Some`).
    /// `index_var` names the runtime `usize` loop variable holding the 1-based
    /// position of the placeholder being built.
    fn runtime_placeholder(&self, index_var: &syn::Ident) -> proc_macro2::TokenStream;

    fn generate_from_row(
        &self,
        struct_name: &syn::Ident,
        fields: &[GeormField],
    ) -> proc_macro2::TokenStream {
        let field_idents: Vec<&syn::Ident> = fields.iter().map(|f| &f.ident).collect();
        let field_names: Vec<String> = fields.iter().map(|f| f.ident.to_string()).collect();
        let row = self.row_type();
        quote! {
            impl<'r> ::sqlx::FromRow<'r, #row> for #struct_name {
                fn from_row(row: &'r #row) -> ::sqlx::Result<Self> {
                    use ::sqlx::Row;
                    Ok(Self {
                        #(#field_idents: row.try_get(#field_names)?),*
                    })
                }
            }
        }
    }

    fn generate_find_all(&self, table: &str) -> proc_macro2::TokenStream {
        let find_string = format!("SELECT * FROM {table}");
        let database = self.database_type();
        quote! {
            async fn find_all<'e, E>(mut executor: E) -> ::sqlx::Result<Vec<Self>>
            where
                E: ::sqlx::Executor<'e, Database = #database>
            {
                ::sqlx::query_as!(Self, #find_string).fetch_all(executor).await
            }
        }
    }

    fn generate_find(&self, table: &str, id: &IdType) -> proc_macro2::TokenStream {
        let database = self.database_type();
        match id {
            IdType::Simple {
                field_name,
                field_type,
            } => {
                let placeholder = self.placeholder(1);
                let find_string =
                    format!("SELECT * FROM {table} WHERE {field_name} = {placeholder}");
                quote! {
                    async fn find<'e, E>(mut executor: E, id: &#field_type) -> ::sqlx::Result<Option<Self>>
                    where
                        E: ::sqlx::Executor<'e, Database = #database>
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
                    .map(|(i, field)| format!("{} = {}", field.name, self.placeholder(i + 1)))
                    .collect::<Vec<String>>()
                    .join(" AND ");
                let id_members: Vec<syn::Ident> =
                    fields.iter().map(|field| field.name.clone()).collect();
                let find_string = format!("SELECT * FROM {table} WHERE {id_match_string}");
                quote! {
                    async fn find<'e, E>(mut executor: E, id: &#field_type) -> ::sqlx::Result<Option<Self>>
                    where
                        E: ::sqlx::Executor<'e, Database = #database>
                    {
                        ::sqlx::query_as!(Self, #find_string, #(id.#id_members),*)
                            .fetch_optional(executor)
                            .await
                    }
                }
            }
        }
    }

    fn generate_create(&self, table: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
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
        let placeholders: Vec<String> = (1..=insert_fields.len())
            .map(|i| self.placeholder(i))
            .collect();
        let query = format!(
            "INSERT INTO {table} ({}) VALUES ({}) RETURNING *",
            field_names.join(", "),
            placeholders.join(", ")
        );
        let database = self.database_type();
        quote! {
            async fn create<'e, E>(&self, mut executor: E) -> ::sqlx::Result<Self>
            where
                E: ::sqlx::Executor<'e, Database = #database>
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

    fn generate_update(&self, table: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
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
            .map(|(i, field)| format!("{} = {}", field.ident, self.placeholder(i + 1)))
            .collect();
        let where_clauses: Vec<String> = id_fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                format!(
                    "{} = {}",
                    field.ident,
                    self.placeholder(update_fields.len() + i + 1)
                )
            })
            .collect();
        let query = format!(
            "UPDATE {table} SET {} WHERE {} RETURNING *",
            set_clauses.join(", "),
            where_clauses.join(" AND ")
        );
        let database = self.database_type();
        quote! {
            async fn update<'e, E>(&self, mut executor: E) -> ::sqlx::Result<Self>
            where
                E: ::sqlx::Executor<'e, Database = #database>
            {
                ::sqlx::query_as!(
                    Self,
                    #query,
                    #(self.#update_idents),*,
                    #(self.#id_idents),*
                )
                .fetch_one(executor)
                .await
            }
        }
    }

    fn generate_upsert(
        &self,
        table: &str,
        fields: &[GeormField],
        id: &IdType,
    ) -> proc_macro2::TokenStream {
        let fields: Vec<&GeormField> = fields
            .iter()
            .filter(|field| !matches!(field.generated_type, GeneratedType::Always))
            .collect();
        let inputs: Vec<String> = (1..=fields.len()).map(|i| self.placeholder(i)).collect();
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
        let database = self.database_type();

        quote! {
            async fn upsert<'e, E>(&self, mut executor: E) -> ::sqlx::Result<Self>
            where
                E: ::sqlx::Executor<'e, Database = #database>
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

    fn generate_delete(&self, table: &str, id: &IdType) -> proc_macro2::TokenStream {
        let where_clause = match id {
            IdType::Simple { field_name, .. } => {
                format!("{} = {}", field_name, self.placeholder(1))
            }
            IdType::Composite { fields, .. } => fields
                .iter()
                .enumerate()
                .map(|(i, field)| format!("{} = {}", field.name, self.placeholder(i + 1)))
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
        let database = self.database_type();
        quote! {
            async fn delete<'e, E>(&self, mut executor: E) -> ::sqlx::Result<u64>
            where
                E: ::sqlx::Executor<'e, Database = #database>
            {
                Self::delete_by_id(executor, &self.get_id()).await
            }

            async fn delete_by_id<'e, E>(mut executor: E, id: &#id_type) -> ::sqlx::Result<u64>
            where
                E: ::sqlx::Executor<'e, Database = #database>
            {
                let rows_affected = ::sqlx::query!(#delete_string, #query_args)
                    .execute(executor)
                    .await?
                    .rows_affected();
                Ok(rows_affected)
            }
        }
    }

    /// Wraps a single-parameter relation-lookup query (field-level
    /// `#[georm(relation = ...)]`, struct-level `one_to_one`/`one_to_many`,
    /// and many-to-many joins) in an async method with the right executor
    /// bound. `query` must already contain this dialect's placeholder syntax
    /// (built via [`SqlDialect::placeholder`]).
    fn generate_relation_lookup(
        &self,
        function: &syn::Ident,
        entity: &syn::Type,
        query: &str,
        arg: &proc_macro2::TokenStream,
        fetch: &FetchKind,
    ) -> proc_macro2::TokenStream {
        let database = self.database_type();
        let (return_type, fetch_method) = match fetch {
            FetchKind::One => (quote! { #entity }, quote! { fetch_one }),
            FetchKind::Optional => (quote! { Option<#entity> }, quote! { fetch_optional }),
            FetchKind::Many => (quote! { Vec<#entity> }, quote! { fetch_all }),
        };
        quote! {
            pub async fn #function<'e, E>(&self, mut executor: E) -> ::sqlx::Result<#return_type>
            where
                E: ::sqlx::Executor<'e, Database = #database>
            {
                // Bound to a place rather than passed inline: sqlx's SQLite
                // query macros need the bind argument to outlive the
                // temporary produced by expressions like `self.get_id()`.
                let arg = #arg;
                ::sqlx::query_as!(#entity, #query, arg).#fetch_method(executor).await
            }
        }
    }
}
