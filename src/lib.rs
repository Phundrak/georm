//! # Georm
//!
//! ## Introduction
//!
//! Georm is a simple, opinionated SQLx ORM for PostgreSQL.
//!
//! To automatically implement the `Georm` trait, you need at least:
//! - to derive the `Georm` and `sqlx::FromRow` traits
//! - use the `georm` proc-macro to indicate the table in which your entity
//!   lives
//! - use the `georm` proc-macro again to indicate which field of your struct is
//!   the identifier of your entity.
//!
//! ## Simple usage
//! Here is a minimal use of Georm with a struct:
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(table = "users")]
//! pub struct User {
//!     #[georm(id)]
//!     id: i32,
//!     username: String,
//!     hashed_password: String,
//! }
//! ```
//!
//! The `User` type will now have access to all the functions declared in the
//! `Georm` trait.
//!
//! ## One-to-one relationships
//!
//! You can then create relationships between different entities. For instance,
//! you can use an identifier of another entity as a link to that other entity.
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(table = "profiles")]
//! pub struct Profile {
//!     #[georm(id)]
//!     id: i32,
//!     #[georm(
//!         relation = {
//!             entity = User,
//!             name = "user",
//!             table = "users",
//!             remote_id = "id",
//!             nullable = false
//!         })
//!     ]
//!     user_id: i32,
//!     display_name: String,
//! }
//! ```
//!
//! This will give access to the `Profile::get_user(&self, pool: &sqlx::PgPool)
//! -> User` method.
//!
//! Here is an explanation of what these different values mean:
//!
//! | Value Name | Explanation                                                                              | Default value |
//! |------------|------------------------------------------------------------------------------------------|---------------|
//! | entity     | Rust type of the entity found in the database                                            | N/A           |
//! | name       | Name of the remote entity within the local entity; generates a method named `get_{name}` | N/A           |
//! | table      | Database table where the entity is stored                                                | N/A           |
//! | remote_id  | Name of the column serving as the identifier of the entity                               | `"id"`        |
//! | nullable   | Whether the relationship can be broken                                                   | `false`       |
//!
//! Note that in this instance, the `remote_id` and `nullable` values can be
//! omitted as this is their default value. This below is a strict equivalent:
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(table = "profiles")]
//! pub struct Profile {
//!     #[georm(id)]
//!     id: i32,
//!     #[georm(relation = { entity = User, table = "users", name = "user" })]
//!     user_id: i32,
//!     display_name: String,
//! }
//! ```
//!
//! But what if I have a one-to-one relationship with another entity and
//! my current entity holds no data to reference that other identity? No
//! worries, there is another way to declare such relationships.
//!
//! ```ignore
//! #[georm(
//!     one_to_one = [{
//!         name = "profile",
//!         remote_id = "user_id",
//!         table = "profiles",
//!         entity = User
//!     }]
//! )]
//! struct User {
//!     #[georm(id)]
//!     id: i32,
//!     username: String,
//!     hashed_password: String,
//! }
//! ```
//!
//! We now have access to the method `User::get_profile(&self, &pool:
//! sqlx::PgPool) -> Option<User>`.
//!
//! Here is an explanation of the values of `one_to_many`:
//!
//! | Value Name | Explanaion                                                                               | Default Value |
//! |------------|------------------------------------------------------------------------------------------|---------------|
//! | entity     | Rust type of the entity found in the database                                            | N/A           |
//! | name       | Name of the remote entity within the local entity; generates a method named `get_{name}` | N/A           |
//! | table      | Database table where the entity is stored                                                | N/A           |
//! | remote_id  | Name of the column serving as the identifier of the entity                               | `"id"`        |
//!
//! ## One-to-many relationships
//!
//! Sometimes, our entity is the one being referenced to by multiple entities,
//! but we have no internal reference to these remote entities in our local
//! entity. Fortunately, we have a way to indicate to Georm how to find these.
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(table = "posts")]
//! struct Post {
//!     #[georm(id)]
//!     id: i32,
//!     #[georm(relation = { entity = User, table = "users", name = "user" })]
//!     author_id: i32,
//!     content: String
//! }
//!
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(
//!     table = "users",
//!     one_to_many = [{
//!         entity    = Post,
//!         name      = "posts",
//!         table     = "posts",
//!         remote_id = "author_id"
//!     }]
//! )]
//! struct User {
//!     #[georm(id)]
//!     id: i32,
//!     username: String,
//!     hashed_password: String
//! }
//! ```
//!
//! As we’ve seen earlier, the struct `Post` has access to the method
//! `Post::get_user(&self, pool: &sqlx::PgPool) -> User` thanks to the
//! proc-macro used on `author_id`. However, `User` now has also access to
//! `User::get_posts(&self, pool: &sqlx::PgPool) -> Vec<Post>`. And as you can
//! see, `one_to_many` is an array, meaning you can define several one-to-many
//! relationships for `User`.
//!
//! Here is an explanation of the values of `one_to_many`:
//!
//! | Value Name | Explanaion                                                                               | Default Value |
//! |------------|------------------------------------------------------------------------------------------|---------------|
//! | entity     | Rust type of the entity found in the database                                            | N/A           |
//! | name       | Name of the remote entity within the local entity; generates a method named `get_{name}` | N/A           |
//! | table      | Database table where the entity is stored                                                | N/A           |
//! | remote_id  | Name of the column serving as the identifier of the entity                               | `"id"`        |
//!
//! As with one-to-one relationships, `remote_id` is optional. The following
//! `User` struct is strictly equivalent.
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(
//!     table = "users",
//!     one_to_many = [{ entity = Post, name = "posts", table = "posts" }]
//! )]
//! struct User {
//!     #[georm(id)]
//!     id: i32,
//!     username: String,
//!     hashed_password: String
//! }
//! ```
//!
//! ## Many-to-many relationships
//!
//! Many-to-many relationships between entities A and entities B with Georm rely
//! on a third table which refers to both. For instance, the following SQL code
//! describes a many-to-many relationship between books and book genre.
//!
//! ```sql
//! CREATE TABLE books (
//!     id SERIAL PRIMARY KEY,
//!     title VARCHAR(100) NOT NULL
//! );
//!
//! CREATE TABLE genres (
//!     id SERIAL PRIMARY KEY,
//!     name VARCHAR(100) NOT NULL
//! );
//!
//! CREATE TABLE books_genres (
//!     book_id INT NOT NULL,
//!     genre_id INT NOT NULL,
//!     PRIMARY KEY (book_id, genre_id),
//!     FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
//!     FOREIGN KEY (genre_id) REFERENCES genres(id) ON DELETE CASCADE
//! );
//! ```
//!
//! The table `books_genres` is the one defining the many-to-many relationship
//! between the table `books` and the table `genres`. With Georm, this gives us
//! the following code:
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(
//!     table = "books",
//!     many_to_many = [{
//!         name = "genres",
//!         entity = Genre,
//!         table = "genres",
//!         remote_id = "id",
//!         link = { table = "books_genres", from = "book_id", to = "genre_id" }
//!     }]
//! )]
//! struct Book {
//!     #[georm(id)]
//!     id: i32,
//!     title: String
//! }
//!
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(
//!     table = "genres",
//!     many_to_many = [{
//!         entity = Book,
//!         name = "books",
//!         table = "books",
//!         remote_id = "id",
//!         link = { table = "books_genres", from = "genre_id", to = "book_id" }
//!     }]
//! )]
//! struct Genre {
//!     #[georm(id)]
//!     id: i32,
//!     name: String
//! }
//! ```
//!
//! This generates two methods:
//! - `Book::get_genres(&self, pool: &sqlx::PgPool) -> Vec<Genre>`
//! - `Genre::get_books(&self, pool: &sqlx::PgPool) -> Vec<Book>`
//!
//! As you can see, `many_to_many` is also an array, meaning we can define
//! several many-to-many relationships for the same struct.
//!
//! Here is an explanation of the values behind `many_to_many`:
//!
//! | Value Name | Explanation                                                                              | Default value |
//! |------------|------------------------------------------------------------------------------------------|---------------|
//! | entity     | Rust type of the entity found in the database                                            | N/A           |
//! | name       | Name of the remote entity within the local entity; generates a method named `get_{name}` | N/A           |
//! | table      | Database table where the entity is stored                                                | N/A           |
//! | remote_id  | Name of the column serving as the identifier of the entity                               | `"id"`        |
//! | link.table | Name of the many-to-many relationship table                                              | N/A           |
//! | link.from  | Column of the linking table referring to this entity                                     | N/A           |
//! | link.to    | Column of the linking table referring to the remote entity                               | N/A           |
//!
//! ## Defaultable Fields
//!
//! Georm supports defaultable fields for entities where some fields have database
//! defaults or are auto-generated (like serial IDs). When you mark fields as
//! `defaultable`, Georm generates a companion struct that makes these fields
//! optional during entity creation.
//!
//! ```ignore
//! #[derive(sqlx::FromRow, Georm)]
//! #[georm(table = "posts")]
//! pub struct Post {
//!     #[georm(id, defaultable)]
//!     id: i32,                    // Auto-generated serial
//!     title: String,              // Required field
//!     #[georm(defaultable)]
//!     published: bool,            // Has database default
//!     #[georm(defaultable)]
//!     created_at: chrono::DateTime<chrono::Utc>, // Has database default
//!     author_id: i32,             // Required field
//! }
//! ```
//!
//! This generates a `PostDefault` struct where defaultable fields become `Option<T>`:
//!
//! ```ignore
//! // Generated automatically by the macro
//! pub struct PostDefault {
//!     pub id: Option<i32>,        // Can be None for auto-generation
//!     pub title: String,          // Required field stays the same
//!     pub published: Option<bool>, // Can be None to use database default
//!     pub created_at: Option<chrono::DateTime<chrono::Utc>>, // Can be None
//!     pub author_id: i32,         // Required field stays the same
//! }
//!
//! impl Defaultable<i32, Post> for PostDefault {
//!     async fn create(&self, pool: &sqlx::PgPool) -> sqlx::Result<Post>;
//! }
//! ```
//!
//! ### Usage Example
//!
//! ```ignore
//! use georm::{Georm, Defaultable};
//!
//! // Create a post with some fields using database defaults
//! let post_default = PostDefault {
//!     id: None,                   // Let database auto-generate
//!     title: "My Blog Post".to_string(),
//!     published: None,            // Use database default (e.g., false)
//!     created_at: None,           // Use database default (e.g., NOW())
//!     author_id: 42,
//! };
//!
//! // Create the entity in the database
//! let created_post = post_default.create(&pool).await?;
//! println!("Created post with ID: {}", created_post.id);
//! ```
//!
//! ### Rules and Limitations
//!
//! - **Option fields cannot be marked as defaultable**: If a field is already
//!   `Option<T>`, you cannot mark it with `#[georm(defaultable)]`. This prevents
//!   `Option<Option<T>>` types.
//! - **Field visibility is preserved**: The generated defaultable struct maintains
//!   the same field visibility (`pub`, `pub(crate)`, private) as the original struct.
//! - **ID fields can be defaultable**: It's common to mark ID fields as defaultable
//!   when they are auto-generated serials in PostgreSQL.
//! - **Only generates when needed**: The defaultable struct is only generated if
//!   at least one field is marked as defaultable.
//!
//! ## Limitations
//! ### Database
//!
//! For now, Georm is limited to PostgreSQL. Other databases may be supported in
//! the future, such as Sqlite or MySQL, but that is not the case yet.
//!
//! ## Identifiers
//!
//! Identifiers, or primary keys from the point of view of the database, may
//! only be simple types recognized by SQLx. They also cannot be arrays, and
//! optionals are only supported in one-to-one relationships when explicitly
//! marked as nullables.

pub use georm_macros::Georm;

mod georm;
pub use georm::Georm;
mod defaultable;
pub use defaultable::Defaultable;
