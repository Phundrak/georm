<h1 align="center">Georm</h1>
<div align="center">
 <strong>
   A simple, opinionated SQLx ORM for PostgreSQL
 </strong>
</div>

<br/>

<div align="center">
  <!-- Github Actions -->
  <a href="https://github.com/phundrak/georm/actions/workflows/ci.yaml?query=branch%3Amain">
    <img src="https://img.shields.io/github/actions/workflow/status/phundrak/georm/ci.yaml?branch=main&style=flat-square" alt="actions status" /></a>
  <!-- Version -->
  <a href="https://crates.io/crates/georm">
    <img src="https://img.shields.io/crates/v/georm.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Discord -->
  <!-- Docs -->
  <a href="https://docs.rs/georm">
  <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
</div>

<div align="center">
  <h4>What is Georm?</h4>
</div>

Georm is a quite simple ORM built around
[SQLx](https://crates.io/crates/sqlx) that gives access to a few
useful functions when interacting with a database, implementing
automatically the most basic SQL interactions you’re tired of writing.

<div align="center">
  <h4>Why is Georm?</h4>
</div>

I wanted an ORM that’s easy and straightforward to use. I am aware
some other projects exist, such as
[SeaORM](https://www.sea-ql.org/SeaORM/), but they generally don’t fit
my needs and/or my wants of a simple interface. I ended up writing the
ORM I wanted to use.

<div align="center">
  <h4>How is Georm?</h4>
</div>

I use it in a few projects, and I’m quite happy with it right now. But
of course, I’m open to constructive criticism and suggestions!

<div align="center">
  <h4>How can I use it?</h4>
</div>

Georm works with SQLx, but does not re-export it itself. To get
started, install both Georm and SQLx in your Rust project:

```sh
cargo add sqlx --features postgres,macros # and any other feature you might want
cargo add georm
```

As Georm relies heavily on the macro
[`query_as!`](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html),
the `macros` feature is not optional. Declare your tables in your
Postgres database (you may want to use SQLx’s `migrate` feature for
this), and then declare their equivalent in Rust.

```sql
CREATE TABLE biographies (
    id SERIAL PRIMARY KEY,
    content TEXT NOT NULL
);

CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    biography_id INT,
    FOREIGN KEY (biography_id) REFERENCES biographies(id)
);
```

```rust
pub struct Author {
    pub id: i32,
    pub name: String,
}
```

To link a struct to a table in your database, derive the
`sqlx::FromRow` and the `georm::Georm` traits.
```rust
#[derive(sqlx::FromRow, Georm)]
pub struct Author {
    pub id: i32,
    pub name: String,
}
```

Now, indicate with the `georm` proc-macro which table they refer to.
```rust
#[derive(sqlx::FromRow, Georm)]
#[georm(table = "authors")]
pub struct Author {
    pub id: i32,
    pub name: String,
}
```

Finally, indicate with the same proc-macro which field of your struct
is the primary key in your database.
```rust
#[derive(sqlx::FromRow, Georm)]
#[georm(table = "authors")]
pub struct Author {
    #[georm(id)]
    pub id: i32,
    pub name: String,
}
```

Congratulations, your struct `Author` now has access to all the
functions described in the `Georm` trait!

<div align="center">
  <h4>Entity relationship</h4>
</div>

It is possible to implement one-to-one, one-to-many, and many-to-many
relationships with Georm. This is a quick example of how a struct with
several relationships of different types may be declared:
```rust
#[derive(sqlx::FromRow, Georm)]
#[georm(
    table = "books",
    one_to_many = [
        { name = "reviews", remote_id = "book_id", table = "reviews", entity = Review }
    ],
    many_to_many = [{
        name = "genres",
        table = "genres",
        entity = Genre,
        link = { table = "book_genres", from = "book_id", to = "genre_id" }
    }]
)]
pub struct Book {
    #[georm(id)]
    ident: i32,
    title: String,
    #[georm(relation = {entity = Author, table = "authors", name = "author"})]
    author_id: i32,
}
```

To read more about these features, you can refer to the [online
documentation](https://docs.rs/sqlx/latest/georm/).
