use crate::georm::IdType;
use crate::georm::sql::{self, SqlDialect};

pub fn generate_delete_query(table: &str, id: &IdType) -> proc_macro2::TokenStream {
    sql::DIALECT.generate_delete(table, id)
}
