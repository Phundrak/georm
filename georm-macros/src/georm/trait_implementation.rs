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

fn generate_find_query(table: &str, id: &GeormField) -> proc_macro2::TokenStream {
    let find_string = format!("SELECT * FROM {table} WHERE {} = $1", id.ident);
    let ty = &id.ty;
    quote! {
        async fn find(pool: &::sqlx::PgPool, id: &#ty) -> ::sqlx::Result<Option<Self>> {
            ::sqlx::query_as!(Self, #find_string, id)
                .fetch_optional(pool)
                .await
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
    id: &GeormField,
) -> proc_macro2::TokenStream {
    let mut fields: Vec<&GeormField> = fields.iter().filter(|f| !f.id).collect();
    let update_columns = fields
        .iter()
        .enumerate()
        .map(|(i, &field)| format!("{} = ${}", field.ident, i + 1))
        .collect::<Vec<String>>()
        .join(", ");
    let update_string = format!(
        "UPDATE {table} SET {update_columns} WHERE {} = ${} RETURNING *",
        id.ident,
        fields.len() + 1
    );
    fields.push(id);
    let field_idents: Vec<_> = fields.iter().map(|f| f.ident.clone()).collect();
    quote! {
        async fn update(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<Self> {
            ::sqlx::query_as!(
                Self,
                #update_string,
                #(self.#field_idents),*
            )
            .fetch_one(pool)
            .await
        }
    }
}

fn generate_delete_query(table: &str, id: &GeormField) -> proc_macro2::TokenStream {
    let delete_string = format!("DELETE FROM {table} WHERE {} = $1", id.ident);
    let ty = &id.ty;
    quote! {
        async fn delete_by_id(pool: &::sqlx::PgPool, id: &#ty) -> ::sqlx::Result<u64> {
            let rows_affected = ::sqlx::query!(#delete_string, id)
                .execute(pool)
                .await?
                .rows_affected();
            Ok(rows_affected)
        }

        async fn delete(&self, pool: &::sqlx::PgPool) -> ::sqlx::Result<u64> {
            Self::delete_by_id(pool, self.get_id()).await
        }
    }
}

fn generate_get_id(id: &GeormField) -> proc_macro2::TokenStream {
    let ident = &id.ident;
    let ty = &id.ty;
    quote! {
        fn get_id(&self) -> &#ty {
            &self.#ident
        }
    }
}

pub fn derive_trait(
    ast: &syn::DeriveInput,
    table: &str,
    fields: &[GeormField],
    id: &GeormField,
) -> proc_macro2::TokenStream {
    let ty = &id.ty;

    // define impl variables
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    // generate
    let get_all = generate_find_all_query(table);
    let get_id = generate_get_id(id);
    let find_query = generate_find_query(table, id);
    let create_query = generate_create_query(table, fields);
    let update_query = generate_update_query(table, fields, id);
    let delete_query = generate_delete_query(table, id);
    quote! {
        impl #impl_generics Georm<#ty> for #ident #type_generics #where_clause {
            #get_all
            #get_id
            #find_query
            #create_query
            #update_query
            #delete_query
        }
    }
}
