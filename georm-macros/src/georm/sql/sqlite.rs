use super::SqlDialect;
use quote::quote;

pub struct SqliteDialect;

impl SqlDialect for SqliteDialect {
    fn placeholder(&self, _index: usize) -> String {
        // SQLite accepts unnumbered `?` placeholders resolved in positional
        // (left-to-right) order, which matches the order Georm binds values in.
        "?".to_string()
    }

    fn database_type(&self) -> proc_macro2::TokenStream {
        quote! { ::sqlx::Sqlite }
    }

    fn row_type(&self) -> proc_macro2::TokenStream {
        quote! { ::sqlx::sqlite::SqliteRow }
    }

    fn runtime_placeholder(&self, index_var: &syn::Ident) -> proc_macro2::TokenStream {
        quote! {
            {
                let _ = #index_var;
                "?".to_string()
            }
        }
    }
}
