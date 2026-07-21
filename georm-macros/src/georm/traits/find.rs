use crate::georm::IdType;
use crate::georm::sql::{self, SqlDialect};

pub fn generate_find_all_query(table: &str) -> proc_macro2::TokenStream {
    sql::DIALECT.generate_find_all(table)
}

pub fn generate_find_query(table: &str, id: &IdType) -> proc_macro2::TokenStream {
    sql::DIALECT.generate_find(table, id)
}
