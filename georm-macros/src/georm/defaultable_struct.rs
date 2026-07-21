//! This module creates the defaultable version of a structured derived with
//! Georm. It creates a new struct named `<StructName>Default` where the fields
//! marked as defaultable become an `Option<T>`, where `T` is the initial type
//! of the field.
//!
//! The user does not have to mark a field defaultable if the field already has
//! a type `Option<T>`. It is intended only for fields marked as `NOT NULL` in
//! the database, but not required when creating the entity due to a `DEFAULT`
//! or something similar. The type `<StructName>Default` implements the
//! `Defaultable` trait.

use crate::georm::ir::GeneratedType;
use crate::georm::sql::{self, SqlDialect};

use super::ir::{GeormField, GeormStructAttributes};
use quote::quote;

fn create_defaultable_field(field: &GeormField) -> proc_macro2::TokenStream {
    let ident = &field.ident;
    let ty = &field.ty;
    let vis = &field.field.vis;

    // If the field is marked as defaultable, wrap it in Option<T>
    // Otherwise, keep the original type
    let field_type = if field.is_defaultable_behavior() {
        quote! { Option<#ty> }
    } else {
        quote! { #ty }
    };

    quote! {
        #vis #ident: #field_type
    }
}

fn generate_defaultable_trait_impl(
    struct_name: &syn::Ident,
    defaultable_struct_name: &syn::Ident,
    struct_attrs: &GeormStructAttributes,
    fields: &[GeormField],
) -> proc_macro2::TokenStream {
    let table = &struct_attrs.table;

    // Find the ID field
    let id_field = fields
        .iter()
        .find(|field| field.is_id)
        .expect("Must have an ID field");
    let id_type = &id_field.ty;

    // Remove always generated fields
    let fields: Vec<&GeormField> = fields
        .iter()
        .filter(|field| !matches!(field.generated_type, GeneratedType::Always))
        .collect();

    // Separate defaultable and non-defaultable fields
    let non_defaultable_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.is_defaultable_behavior())
        .collect();
    let defaultable_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.is_defaultable_behavior())
        .collect();

    // Build static parts for non-defaultable fields
    let static_field_names: Vec<String> = non_defaultable_fields
        .iter()
        .map(|f| f.ident.to_string())
        .collect();
    let static_field_idents: Vec<&syn::Ident> =
        non_defaultable_fields.iter().map(|f| &f.ident).collect();

    // Generate field checks for defaultable fields
    let mut field_checks = Vec::new();
    let mut bind_checks = Vec::new();

    for field in &defaultable_fields {
        let field_name = field.ident.to_string();
        let field_ident = &field.ident;

        field_checks.push(quote! {
            if self.#field_ident.is_some() {
                dynamic_fields.push(#field_name);
            }
        });

        bind_checks.push(quote! {
            if let Some(ref value) = self.#field_ident {
                query_builder = query_builder.bind(value);
            }
        });
    }

    let database = sql::DIALECT.database_type();
    let index_var = syn::Ident::new("i", proc_macro2::Span::call_site());
    let placeholder_expr = sql::DIALECT.runtime_placeholder(&index_var);

    quote! {
        impl ::georm::Defaultable<#id_type, #struct_name> for #defaultable_struct_name {
            async fn create<'e, E>(&self, mut executor: E) -> ::sqlx::Result<#struct_name>
            where
                E: ::sqlx::Executor<'e, Database = #database>
            {
                let mut dynamic_fields = Vec::new();

                #(#field_checks)*

                let mut all_fields = vec![#(#static_field_names),*];
                all_fields.extend(dynamic_fields);

                let placeholders: Vec<String> = (1..=all_fields.len())
                    .map(|#index_var| #placeholder_expr)
                    .collect();

                let query = format!(
                    "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
                    #table,
                    all_fields.join(", "),
                    placeholders.join(", ")
                );

                let mut query_builder = ::sqlx::query_as::<_, #struct_name>(&query);

                // Bind non-defaultable fields first
                #(query_builder = query_builder.bind(&self.#static_field_idents);)*

                // Then bind defaultable fields that have values
                #(#bind_checks)*

                query_builder.fetch_one(executor).await
            }
        }
    }
}

pub fn derive_defaultable_struct(
    ast: &syn::DeriveInput,
    struct_attrs: &GeormStructAttributes,
    fields: &[GeormField],
) -> proc_macro2::TokenStream {
    // Only generate if there are defaultable fields
    if fields.iter().all(|field| !field.is_defaultable_behavior()) {
        return quote! {};
    }

    let struct_name = &ast.ident;
    let vis = &ast.vis;
    let defaultable_struct_name = quote::format_ident!("{}Default", struct_name);

    let defaultable_fields: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .flat_map(|field| match field.generated_type {
            GeneratedType::Always => None,
            _ => Some(create_defaultable_field(field)),
        })
        .collect();

    let trait_impl = generate_defaultable_trait_impl(
        struct_name,
        &defaultable_struct_name,
        struct_attrs,
        fields,
    );

    quote! {
        #vis struct #defaultable_struct_name {
            #(#defaultable_fields),*
        }

        #trait_impl
    }
}
