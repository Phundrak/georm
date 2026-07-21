#![cfg(feature = "sqlite")]

use georm::Georm;

#[derive(Debug, Georm, PartialEq, Eq, Default)]
#[georm(
    table = "biographies",
    one_to_one = [{
        name = "author", remote_id = "biography_id", table = "authors", entity = Author
    }]
)]
pub struct Biography {
    #[georm(id)]
    pub id: i64,
    pub content: String,
}

#[derive(Debug, Georm, PartialEq, Eq, Default)]
#[georm(table = "authors")]
pub struct Author {
    #[georm(id)]
    pub id: i64,
    pub name: String,
    #[georm(relation = {entity = Biography, table = "biographies", name = "biography", nullable = true})]
    pub biography_id: Option<i64>,
}

impl PartialOrd for Author {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Author {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

// UserRole (composite key, chrono::DateTime<Utc> column) is deliberately not
// ported here yet: SQLite's compile-time query macros infer TEXT columns as
// `String`, not `chrono::DateTime<Utc>` — unlike Postgres's TIMESTAMPTZ, which
// maps directly. Fixing this needs per-column `query_as!` type overrides
// (`"assigned_at as \"assigned_at: DateTime<Utc>\""`) threaded through the
// dialect's generated queries, which is follow-up work, not part of this
// first SQLite slice. See tests/sqlite/composite_key.rs (not yet added).
