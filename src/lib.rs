//! # Georm
//!
//! A simple, type-safe PostgreSQL ORM built on SQLx with zero runtime overhead.
//!
//! ## Quick Start
//!
//! ```ignore
//! use georm::Georm;
//!
//! // Note: No need to derive FromRow - Georm generates it automatically
//! #[derive(Georm)]
//! #[georm(table = "users")]
//! pub struct User {
//!     #[georm(id)]
//!     id: i32,
//!     username: String,
//!     email: String,
//! }
//!
//! // Use generated methods
//! let user = User::find(&pool, &1).await?;      // Static method
//! let all_users = User::find_all(&pool).await?; // Static method
//! user.update(&pool).await?;                    // Instance method
//! ```
//!
//! ## Core CRUD Operations
//!
//! ### Static Methods (called on the struct type)
//! - `Entity::find(pool, &id)` - Find by primary key, returns `Option<Entity>`
//! - `Entity::find_all(pool)` - Get all records, returns `Vec<Entity>`
//! - `Entity::delete_by_id(pool, &id)` - Delete by ID, returns affected row count
//!
//! ### Instance Methods (called on entity objects)
//! - `entity.create(pool)` - Insert new record, returns created entity with database-generated values
//! - `entity.update(pool)` - Update existing record, returns updated entity with fresh database state
//! - `entity.create_or_update(pool)` - True PostgreSQL upsert using `ON CONFLICT`, returns final entity
//! - `entity.delete(pool)` - Delete this record, returns affected row count
//! - `entity.get_id()` - Get reference to the entity's ID (`&Id` for simple keys, owned for composite)
//!
//! ```ignore
//! // Static methods
//! let user = User::find(&pool, &1).await?.unwrap();
//! let all_users = User::find_all(&pool).await?;
//! let deleted_count = User::delete_by_id(&pool, &1).await?;
//!
//! // Instance methods
//! let new_user = User { id: 0, username: "alice".to_string(), email: "alice@example.com".to_string() };
//! let created = new_user.create(&pool).await?;        // Returns entity with actual generated ID
//! let updated = created.update(&pool).await?;         // Returns entity with fresh database state
//! let deleted_count = updated.delete(&pool).await?;
//! ```
//!
//! ### PostgreSQL Optimizations
//!
//! Georm leverages PostgreSQL-specific features for performance and reliability:
//!
//! - **RETURNING clause**: All `INSERT` and `UPDATE` operations use `RETURNING *` to capture database-generated values (sequences, defaults, triggers)
//! - **True upserts**: `create_or_update()` uses `INSERT ... ON CONFLICT ... DO UPDATE` for atomic upsert operations
//! - **Prepared statements**: All queries use parameter binding for security and performance
//! - **Compile-time verification**: SQLx macros verify all generated SQL against your database schema at compile time
//!
//! ## Primary Keys and Identifiers
//!
//! ### Simple Primary Keys
//!
//! Primary key fields can have any name (not just "id"):
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "books")]
//! pub struct Book {
//!     #[georm(id)]
//!     ident: i32,  // Custom field name for primary key
//!     title: String,
//! }
//!
//! // Works the same way
//! let book = Book::find(&pool, &1).await?;
//! ```
//!
//! ### Composite Primary Keys
//!
//! Mark multiple fields with `#[georm(id)]` for composite keys:
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "user_roles")]
//! pub struct UserRole {
//!     #[georm(id)]
//!     user_id: i32,
//!     #[georm(id)]
//!     role_id: i32,
//!     assigned_at: chrono::DateTime<chrono::Utc>,
//! }
//! ```
//!
//! This automatically generates a composite ID struct following the `{EntityName}Id` pattern:
//!
//! ```ignore
//! // Generated automatically by the macro
//! pub struct UserRoleId {
//!     pub user_id: i32,
//!     pub role_id: i32,
//! }
//! ```
//!
//! Usage with composite keys:
//!
//! ```ignore
//! // Static methods work with generated ID structs
//! let id = UserRoleId { user_id: 1, role_id: 2 };
//! let user_role = UserRole::find(&pool, &id).await?;
//! UserRole::delete_by_id(&pool, &id).await?;
//!
//! // Instance methods work the same way
//! let role = UserRole { user_id: 1, role_id: 2, assigned_at: chrono::Utc::now() };
//! let created = role.create(&pool).await?;
//! let id = created.get_id(); // Returns owned UserRoleId for composite keys
//! ```
//!
//! ### Composite Key Limitations
//!
//! - **Relationships not supported**: Entities with composite primary keys cannot
//!   yet define relationships (one-to-one, one-to-many, many-to-many)
//! - **ID struct naming**: Generated ID struct follows pattern `{EntityName}Id` (not customizable)
//!
//! ## Defaultable Fields
//!
//! Use `#[georm(defaultable)]` for fields with database defaults or auto-generated values:
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "posts")]
//! pub struct Post {
//!     #[georm(id, defaultable)]
//!     id: i32,                    // Auto-generated serial
//!     title: String,              // Required field
//!     #[georm(defaultable)]
//!     published: bool,            // Has database default
//!     #[georm(defaultable)]
//!     created_at: chrono::DateTime<chrono::Utc>, // DEFAULT NOW()
//!     #[georm(defaultable)]
//!     pub(crate) internal_note: String,          // Field visibility preserved
//!     author_id: i32,             // Required field
//! }
//! ```
//!
//! This generates a companion `PostDefault` struct where defaultable fields become `Option<T>`:
//!
//! ```ignore
//! // Generated automatically by the macro
//! pub struct PostDefault {
//!     pub id: Option<i32>,        // Can be None for auto-generation
//!     pub title: String,          // Required field stays the same
//!     pub published: Option<bool>, // Can be None to use database default
//!     pub created_at: Option<chrono::DateTime<chrono::Utc>>, // Can be None
//!     pub(crate) internal_note: Option<String>,  // Visibility preserved
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
//!     internal_note: Some("Draft".to_string()),
//!     author_id: 42,
//! };
//!
//! // Create the entity in the database (instance method on PostDefault)
//! let created_post = post_default.create(&pool).await?;
//! println!("Created post with ID: {}", created_post.id);
//! ```
//!
//! ### Defaultable Rules and Limitations
//!
//! - **Option fields cannot be marked as defaultable**: If a field is already
//!   `Option<T>`, you cannot mark it with `#[georm(defaultable)]`. This prevents
//!   `Option<Option<T>>` types and causes a compile-time error.
//! - **Field visibility is preserved**: The generated defaultable struct maintains
//!   the same field visibility (`pub`, `pub(crate)`, private) as the original struct.
//! - **ID fields can be defaultable**: It's common to mark ID fields as defaultable
//!   when they are auto-generated serials in PostgreSQL.
//! - **Only generates when needed**: The defaultable struct is only generated if
//!   at least one field is marked as defaultable.
//!
//! ## Relationships
//!
//! Georm supports comprehensive relationship modeling with two approaches: field-level
//! relationships for foreign keys and struct-level relationships for reverse lookups.
//! Each relationship method call executes a separate database query.
//!
//! ### Field-Level Relationships (Foreign Keys)
//!
//! Use the `relation` attribute on foreign key fields to generate lookup methods:
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "posts")]
//! pub struct Post {
//!     #[georm(id)]
//!     id: i32,
//!     title: String,
//!     #[georm(relation = {
//!         entity = Author,        // Target entity type
//!         table = "authors",      // Target table name
//!         name = "author",        // Method name (generates get_author)
//!         remote_id = "id",       // Target table's key column (default: "id")
//!         nullable = false        // Whether relationship can be null (default: false)
//!     })]
//!     author_id: i32,
//! }
//! ```
//!
//! **Generated instance method**: `post.get_author(pool).await? -> sqlx::Result<Author>`
//!
//! For nullable relationships:
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "posts")]
//! pub struct Post {
//!     #[georm(id)]
//!     id: i32,
//!     title: String,
//!     #[georm(relation = {
//!         entity = Category,
//!         table = "categories",
//!         name = "category",
//!         nullable = true         // Allows NULL values
//!     })]
//!     category_id: Option<i32>,
//! }
//! ```
//!
//! **Generated instance method**: `post.get_category(pool).await? -> sqlx::Result<Option<Category>>`
//!
//! Since `remote_id` and `nullable` have default values, this is equivalent:
//!
//! ```ignore
//! #[georm(relation = { entity = Author, table = "authors", name = "author" })]
//! author_id: i32,
//! ```
//!
//! #### Non-Standard Primary Key References
//!
//! Use `remote_id` to reference tables with non-standard primary key names:
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(table = "reviews")]
//! pub struct Review {
//!     #[georm(id)]
//!     id: i32,
//!     #[georm(relation = {
//!         entity = Book,
//!         table = "books",
//!         name = "book",
//!         remote_id = "ident"     // Book uses 'ident' instead of 'id'
//!     })]
//!     book_id: i32,
//!     content: String,
//! }
//! ```
//!
//! #### Field-Level Relationship Attributes
//!
//! | Attribute    | Description                                          | Required | Default |
//! |--------------|------------------------------------------------------|----------|---------|
//! | `entity`     | Target entity type                                   | Yes      | N/A     |
//! | `name`       | Method name (generates `get_{name}`)                 | Yes      | N/A     |
//! | `table`      | Target table name                                    | Yes      | N/A     |
//! | `remote_id`  | Target table's key column                            | No       | `"id"`  |
//! | `nullable`   | Whether relationship can be null                     | No       | `false` |
//!
//! ### Struct-Level Relationships (Reverse Lookups)
//!
//! Define relationships at the struct level to query related entities that reference this entity.
//! These generate separate database queries for each method call.
//!
//! #### One-to-One Relationships
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(
//!     table = "users",
//!     one_to_one = [{
//!         entity = Profile,       // Related entity type
//!         name = "profile",       // Method name (generates get_profile)
//!         table = "profiles",     // Related table name
//!         remote_id = "user_id",  // Foreign key in related table
//!     }]
//! )]
//! pub struct User {
//!     #[georm(id)]
//!     id: i32,
//!     username: String,
//! }
//! ```
//!
//! **Generated instance method**: `user.get_profile(pool).await? -> sqlx::Result<Option<Profile>>`
//!
//! #### One-to-Many Relationships
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(
//!     table = "authors",
//!     one_to_many = [{
//!         entity = Post,          // Related entity type
//!         name = "posts",         // Method name (generates get_posts)
//!         table = "posts",        // Related table name
//!         remote_id = "author_id" // Foreign key in related table
//!     }, {
//!         entity = Comment,       // Multiple relationships allowed
//!         name = "comments",
//!         table = "comments",
//!         remote_id = "author_id"
//!     }]
//! )]
//! pub struct Author {
//!     #[georm(id)]
//!     id: i32,
//!     name: String,
//! }
//! ```
//!
//! **Generated instance methods**:
//! - `author.get_posts(pool).await? -> sqlx::Result<Vec<Post>>`
//! - `author.get_comments(pool).await? -> sqlx::Result<Vec<Comment>>`
//!
//! #### Many-to-Many Relationships
//!
//! For many-to-many relationships, specify the link table that connects the entities:
//!
//! ```sql
//! -- Example schema for books and genres
//! CREATE TABLE books (
//!     id SERIAL PRIMARY KEY,
//!     title VARCHAR(200) NOT NULL
//! );
//!
//! CREATE TABLE genres (
//!     id SERIAL PRIMARY KEY,
//!     name VARCHAR(100) NOT NULL
//! );
//!
//! CREATE TABLE book_genres (
//!     book_id INT NOT NULL REFERENCES books(id),
//!     genre_id INT NOT NULL REFERENCES genres(id),
//!     PRIMARY KEY (book_id, genre_id)
//! );
//! ```
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(
//!     table = "books",
//!     many_to_many = [{
//!         entity = Genre,         // Related entity type
//!         name = "genres",        // Method name (generates get_genres)
//!         table = "genres",       // Related table name
//!         remote_id = "id",       // Primary key in related table (default: "id")
//!         link = {                // Link table configuration
//!             table = "book_genres",  // Join table name
//!             from = "book_id",       // Column referencing this entity
//!             to = "genre_id"         // Column referencing related entity
//!         }
//!     }]
//! )]
//! pub struct Book {
//!     #[georm(id)]
//!     id: i32,
//!     title: String,
//! }
//!
//! #[derive(Georm)]
//! #[georm(
//!     table = "genres",
//!     many_to_many = [{
//!         entity = Book,
//!         name = "books",
//!         table = "books",
//!         link = {
//!             table = "book_genres",
//!             from = "genre_id",      // Note: reversed perspective
//!             to = "book_id"
//!         }
//!     }]
//! )]
//! pub struct Genre {
//!     #[georm(id)]
//!     id: i32,
//!     name: String,
//! }
//! ```
//!
//! **Generated instance methods**:
//! - `book.get_genres(pool).await? -> sqlx::Result<Vec<Genre>>`
//! - `genre.get_books(pool).await? -> sqlx::Result<Vec<Book>>`
//!
//! #### Struct-Level Relationship Attributes
//!
//! | Attribute    | Description                                          | Required | Default |
//! |--------------|------------------------------------------------------|----------|---------|
//! | `entity`     | Target entity type                                   | Yes      | N/A     |
//! | `name`       | Method name (generates `get_{name}`)                 | Yes      | N/A     |
//! | `table`      | Target table name                                    | Yes      | N/A     |
//! | `remote_id`  | Target table's key column                            | No       | `"id"`  |
//! | `link.table` | Join table name (many-to-many only)                  | Yes*     | N/A     |
//! | `link.from`  | Column referencing this entity (many-to-many only)   | Yes*     | N/A     |
//! | `link.to`    | Column referencing target entity (many-to-many only) | Yes*     | N/A     |
//!
//! *Required for many-to-many relationships
//!
//! As with field-level relationships, `remote_id` is optional and defaults to `"id"`:
//!
//! ```ignore
//! #[georm(
//!     table = "users",
//!     one_to_many = [{ entity = Post, name = "posts", table = "posts" }]
//! )]
//! ```
//!
//! #### Complex Relationship Example
//!
//! Here's a comprehensive example showing multiple relationship types:
//!
//! ```ignore
//! #[derive(Georm)]
//! #[georm(
//!     table = "posts",
//!     one_to_many = [{
//!         entity = Comment,
//!         name = "comments",
//!         table = "comments",
//!         remote_id = "post_id"
//!     }],
//!     many_to_many = [{
//!         entity = Tag,
//!         name = "tags",
//!         table = "tags",
//!         link = {
//!             table = "post_tags",
//!             from = "post_id",
//!             to = "tag_id"
//!         }
//!     }]
//! )]
//! pub struct Post {
//!     #[georm(id)]
//!     id: i32,
//!     title: String,
//!     content: String,
//!
//!     // Field-level relationship (foreign key)
//!     #[georm(relation = {
//!         entity = Author,
//!         table = "authors",
//!         name = "author"
//!     })]
//!     author_id: i32,
//!
//!     // Nullable field-level relationship
//!     #[georm(relation = {
//!         entity = Category,
//!         table = "categories",
//!         name = "category",
//!         nullable = true
//!     })]
//!     category_id: Option<i32>,
//! }
//! ```
//!
//! **Generated instance methods**:
//! - `post.get_author(pool).await? -> sqlx::Result<Author>` (from field relation)
//! - `post.get_category(pool).await? -> sqlx::Result<Option<Category>>` (nullable field relation)
//! - `post.get_comments(pool).await? -> sqlx::Result<Vec<Comment>>` (one-to-many)
//! - `post.get_tags(pool).await? -> sqlx::Result<Vec<Tag>>` (many-to-many)
//!
//! ## Error Handling
//!
//! All Georm methods return `sqlx::Result<T>` which can contain:
//!
//! - **Database errors**: Connection issues, constraint violations, etc.
//! - **Not found errors**: When `find()` operations return `None`
//! - **Compile-time errors**: Invalid SQL, type mismatches, schema validation failures
//!
//! ### Compile-Time Validations
//!
//! Georm performs several validations at compile time:
//!
//! ```ignore
//! // ❌ Compile error: No ID field specified
//! #[derive(Georm)]
//! #[georm(table = "invalid")]
//! pub struct Invalid {
//!     name: String,  // Missing #[georm(id)]
//! }
//!
//! // ❌ Compile error: Option<T> cannot be defaultable
//! #[derive(Georm)]
//! #[georm(table = "invalid")]
//! pub struct Invalid {
//!     #[georm(id)]
//!     id: i32,
//!     #[georm(defaultable)]  // Error: would create Option<Option<String>>
//!     optional_field: Option<String>,
//! }
//! ```
//!
//! ## Attribute Reference
//!
//! ### Struct-Level Attributes
//!
//! ```ignore
//! #[georm(
//!     table = "table_name",                   // Required: database table name
//!     one_to_one = [{ /* ... */ }],           // Optional: one-to-one relationships
//!     one_to_many = [{ /* ... */ }],          // Optional: one-to-many relationships
//!     many_to_many = [{ /* ... */ }]          // Optional: many-to-many relationships
//! )]
//! ```
//!
//! ### Field-Level Attributes
//!
//! ```ignore
//! #[georm(id)]                               // Mark as primary key (required on at least one field)
//! #[georm(defaultable)]                      // Mark as defaultable field (database default/auto-generated)
//! #[georm(relation = { /* ... */ })]         // Define foreign key relationship
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Zero runtime overhead**: All SQL is generated at compile time
//! - **No eager loading**: Each relationship method executes a separate query
//! - **Prepared statements**: All queries use parameter binding for optimal performance
//! - **Database round-trips**: CRUD operations use RETURNING clause to minimize round-trips
//! - **No N+1 prevention**: Built-in relationships don't prevent N+1 query patterns
//!
//! ## Limitations
//!
//! ### Database Support
//!
//! Georm is currently limited to PostgreSQL. Other databases may be supported in
//! the future, such as SQLite or MySQL, but that is not the case yet.
//!
//! ### Identifiers
//!
//! Identifiers, or primary keys from the point of view of the database, may
//! be simple types recognized by SQLx or composite keys (multiple fields marked
//! with `#[georm(id)]`). Single primary keys cannot be arrays, and optionals are
//! only supported in one-to-one relationships when explicitly marked as nullables.
//!
//! ### Current Limitations
//!
//! - **Composite key relationships**: Entities with composite primary keys cannot define relationships
//! - **Single table per entity**: No table inheritance or polymorphism support
//! - **No advanced queries**: No complex WHERE clauses or joins beyond relationships
//! - **No eager loading**: Each relationship call is a separate database query
//! - **No field-based queries**: No `find_by_{field_name}` methods generated automatically
//! - **PostgreSQL only**: No support for other database systems
//!
//! ## Generated Code
//!
//! Georm automatically generates:
//! - `sqlx::FromRow` implementation (no need to derive manually)
//! - Composite ID structs for multi-field primary keys
//! - Defaultable companion structs for entities with defaultable fields
//! - Relationship methods for accessing related entities
//! - All CRUD operations with proper PostgreSQL optimizations

pub use georm_macros::Georm;

mod georm;
pub use georm::Georm;
mod defaultable;
pub use defaultable::Defaultable;
