use super::SqlDialect;
use quote::quote;

pub struct PostgresDialect;

impl SqlDialect for PostgresDialect {
    fn placeholder(&self, index: usize) -> String {
        format!("${index}")
    }

    fn database_type(&self) -> proc_macro2::TokenStream {
        quote! { ::sqlx::Postgres }
    }

    fn row_type(&self) -> proc_macro2::TokenStream {
        quote! { ::sqlx::postgres::PgRow }
    }

    fn runtime_placeholder(&self, index_var: &syn::Ident) -> proc_macro2::TokenStream {
        quote! { format!("${}", #index_var) }
    }
}
