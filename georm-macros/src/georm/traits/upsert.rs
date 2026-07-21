use crate::georm::sql::{self, SqlDialect};
use crate::georm::{GeormField, IdType};

pub fn generate_upsert_query(
    table: &str,
    fields: &[GeormField],
    id: &IdType,
) -> proc_macro2::TokenStream {
    sql::DIALECT.generate_upsert(table, fields, id)
}
