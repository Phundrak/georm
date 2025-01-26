//! Creates ORM functionality for ``SQLx`` with `PostgreSQL`.
//!
//! This crate provides the trait implementation `Georm` which
//! generates the following ``SQLx`` queries:
//! - find an entity by id
//!
//!   SQL query: `SELECT * FROM ... WHERE <id> = ...`
//! - insert an entity into the database
//!
//!   SQL query: `INSERT INTO ... (...) VALUES (...) RETURNING *`
//! - update an entity in the database
//!
//!   SQL query: `UPDATE ... SET ... WHERE <id> = ... RETURNING *`
//! - delete an entity from the database using its id or an id
//!   provided by the interface’s user
//!
//!   SQL query: `DELETE FROM ... WHERE <id> = ...`
//! - update an entity or create it if it does not already exist in
//!   the database
//!
//! This macro relies on the trait `Georm` found in the `georm`
//! crate.
//!
//! To use this macro, you need to add it to the derives of the
//! struct. You will also need to define its identifier
//!
//! # Usage
//!
//! Add `#[georm(table = "my_table_name")]` atop of the structure,
//! after the `Georm` derive.
//!
//! ## Entity Identifier
//! You will also need to add `#[georm(id)]` atop of the field of your
//! struct that will be used as the identifier of your entity.
//!
//! ## Column Name
//! If the name of a field does not match the name of its related
//! column, you can use `#[georm(column = "...")]` to specify the
//! correct value.
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "users")]
//! pub struct User {
//!     #[georm(id)]
//!     id: String,
//!     #[georm(column = "name")]
//!     username: String,
//!     created_at: Timestampz,
//!     last_updated: Timestampz,
//! }
//! ```
//!
//! With the example of the `User` struct, this links it to the
//! `users` table of the connected database. It will use `Users.id` to
//! uniquely identify a user entity.
//!
//! # Limitations
//! ## ID
//! For now, only one identifier is supported. It does not have to be
//! a primary key, but it is strongly encouraged to use Georm ID on a
//! unique and non-null column of your database schema.
//!
//! ## Database type
//!
//! For now, only the ``PostgreSQL`` syntax is supported. If you use
//! another database that uses the same syntax, you’re in luck!
//! Otherwise, pull requests to add additional syntaxes are most
//! welcome.

mod georm;
use georm::georm_derive_macro2;

/// Generates GEORM code for Sqlx for a struct.
///
/// # Panics
///
/// May panic if errors arise while parsing and generating code.
#[proc_macro_derive(Georm, attributes(georm))]
pub fn georm_derive_macro(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    georm_derive_macro2(item.into()).unwrap().into()
}
