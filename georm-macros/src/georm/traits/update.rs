use crate::georm::GeormField;
use crate::georm::sql::{self, SqlDialect};

pub fn generate_update_query(table_name: &str, fields: &[GeormField]) -> proc_macro2::TokenStream {
    sql::DIALECT.generate_update(table_name, fields)
}
