//! The SQL backend selected via Cargo feature flags (`postgres` or `sqlite`,
//! mutually exclusive — see the guard in `lib.rs`).

#[cfg(feature = "postgres")]
pub type ActiveDatabase = ::sqlx::Postgres;
#[cfg(feature = "sqlite")]
pub type ActiveDatabase = ::sqlx::Sqlite;
