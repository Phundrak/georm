use crate::georm::GeormField;
use crate::georm::sql::{self, SqlDialect};

pub fn generate_create_query(table_name: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
    sql::DIALECT.generate_create(table_name, fields)
}
