<div align="center">
 <a href="https://github.com/Phundrak/georm">
   <img src="assets/logo.png" alt="Georm logo" width="150px" />
 </a>
</div>

<h1 align="center">Georm</h1>
<div align="center">
 <strong>
   A simple, type-safe SQLx ORM for PostgreSQL
 </strong>
</div>
<br/>

<div align="center">
  <!-- Github Actions -->
  <a href="https://github.com/phundrak/georm/actions/workflows/ci.yaml?query=branch%3Amain">
    <img src="https://img.shields.io/github/actions/workflow/status/phundrak/georm/ci.yaml?branch=main&style=flat-square" alt="actions status" />
  </a>
  <!-- Version -->
  <a href="https://crates.io/crates/georm">
    <img src="https://img.shields.io/crates/v/georm.svg?style=flat-square" alt="Crates.io version" />
  </a>
  <!-- Docs -->
  <a href="https://docs.rs/georm">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" />
  </a>
  <!-- License -->
  <a href="#license">
    <img src="https://img.shields.io/badge/license-MIT%20OR%20GPL--3.0-blue?style=flat-square" alt="License" />
  </a>
</div>

## Overview

Georm is a lightweight, opinionated Object-Relational Mapping (ORM) library built on top of [SQLx](https://crates.io/crates/sqlx) for PostgreSQL. It provides a clean, type-safe interface for common database operations while leveraging SQLx's compile-time query verification.

### Key Features

- **Type Safety**: Compile-time verified SQL queries using SQLx macros
- **Zero Runtime Cost**: No reflection or runtime query building
- **Simple API**: Intuitive derive macros for common operations
- **Relationship Support**: One-to-one, one-to-many, and many-to-many relationships
- **Composite Primary Keys**: Support for multi-field primary keys
- **Defaultable Fields**: Easy entity creation with database defaults and auto-generated values
- **PostgreSQL Native**: Optimized for PostgreSQL features and data types

## Quick Start

### Installation

Add Georm and SQLx to your `Cargo.toml`:

```toml
[dependencies]
sqlx = { version = "0.8.6", features = ["runtime-tokio-rustls", "postgres", "macros"] }
georm = "0.1"
```

### Basic Usage

1. **Define your database schema**:

```sql
CREATE TABLE authors (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL
);

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    published BOOLEAN DEFAULT FALSE,
    author_id INT NOT NULL REFERENCES authors(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

2. **Define your Rust entities**:

```rust
use georm::Georm;

#[derive(Georm)]
#[georm(table = "authors")]
pub struct Author {
    #[georm(id)]
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Georm)]
#[georm(table = "posts")]
pub struct Post {
    #[georm(id)]
    pub id: i32,
    pub title: String,
    pub content: String,
    pub published: bool,
    #[georm(relation = {
        entity = Author,
        table = "authors",
        name = "author"
    })]
    pub author_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

3. **Use the generated methods**:

```rust
use sqlx::PgPool;

async fn example(pool: &PgPool) -> sqlx::Result<()> {
    // Create an author
    let author = Author {
        id: 0, // Will be auto-generated
        name: "Jane Doe".to_string(),
        email: "jane@example.com".to_string(),
    };
    let author = author.create(pool).await?;

    // Create a post
    let post = Post {
        id: 0,
        title: "Hello, Georm!".to_string(),
        content: "This is my first post using Georm.".to_string(),
        published: false,
        author_id: author.id,
        created_at: chrono::Utc::now(),
    };
    let post = post.create(pool).await?;

    // Find all posts
    let all_posts = Post::find_all(pool).await?;

    // Get the post's author
    let post_author = post.get_author(pool).await?;

    println!("Post '{}' by {}", post.title, post_author.name);

    Ok(())
}
```

## Advanced Features

### Composite Primary Keys

Georm supports composite primary keys by marking multiple fields with `#[georm(id)]`:

```rust
#[derive(Georm)]
#[georm(table = "user_roles")]
pub struct UserRole {
    #[georm(id)]
    pub user_id: i32,
    #[georm(id)]
    pub role_id: i32,
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}
```

This automatically generates a composite ID struct:

```rust
// Generated automatically
pub struct UserRoleId {
    pub user_id: i32,
    pub role_id: i32,
}

// Usage
let id = UserRoleId { user_id: 1, role_id: 2 };
let user_role = UserRole::find(pool, &id).await?;
```

**Note**: Relationships are not yet supported for entities with composite primary keys.

### Defaultable Fields

For fields with database defaults or auto-generated values, use the `defaultable` attribute:

```rust
#[derive(Georm)]
#[georm(table = "posts")]
pub struct Post {
    #[georm(id, defaultable)]
    pub id: i32,                    // Auto-generated serial
    pub title: String,
    #[georm(defaultable)]
    pub published: bool,            // Has database default (false)
    #[georm(defaultable)]
    pub created_at: chrono::DateTime<chrono::Utc>, // DEFAULT NOW()
    pub author_id: i32,
}
```

This generates a `PostDefault` struct for easier creation:

```rust
use georm::Defaultable;

let post_default = PostDefault {
    id: None,                       // Let database auto-generate
    title: "My Post".to_string(),
    published: None,                // Use database default
    created_at: None,               // Use database default (NOW())
    author_id: 42,
};

let created_post = post_default.create(pool).await?;
```

### Relationships

Georm supports comprehensive relationship modeling with two approaches: field-level relationships for foreign keys and struct-level relationships for reverse lookups.

#### Field-Level Relationships (Foreign Keys)

Use the `relation` attribute on foreign key fields to generate lookup methods:

```rust
#[derive(Georm)]
#[georm(table = "posts")]
pub struct Post {
    #[georm(id)]
    pub id: i32,
    pub title: String,
    #[georm(relation = {
        entity = Author,        // Target entity type
        table = "authors",      // Target table name
        name = "author",        // Method name (generates get_author)
        remote_id = "id",       // Target table's key column (default: "id")
        nullable = false        // Whether relationship can be null (default: false)
    })]
    pub author_id: i32,
}
```

**Generated method**: `post.get_author(pool).await? -> Author`

For nullable relationships:

```rust
#[derive(Georm)]
#[georm(table = "posts")]
pub struct Post {
    #[georm(id)]
    pub id: i32,
    pub title: String,
    #[georm(relation = {
        entity = Category,
        table = "categories",
        name = "category",
        nullable = true         // Allows NULL values
    })]
    pub category_id: Option<i32>,
}
```

**Generated method**: `post.get_category(pool).await? -> Option<Category>`

#### Struct-Level Relationships (Reverse Lookups)

Define relationships at the struct level to query related entities that reference this entity:

##### One-to-One Relationships

```rust
#[derive(Georm)]
#[georm(
    table = "users",
    one_to_one = [{
        entity = Profile,       // Related entity type
        name = "profile",       // Method name (generates get_profile)
        table = "profiles",     // Related table name
        remote_id = "user_id",  // Foreign key in related table
    }]
)]
pub struct User {
    #[georm(id)]
    pub id: i32,
    pub username: String,
}
```

**Generated method**: `user.get_profile(pool).await? -> Option<Profile>`

##### One-to-Many Relationships

```rust
#[derive(Georm)]
#[georm(
    table = "authors",
    one_to_many = [{
        entity = Post,          // Related entity type
        name = "posts",         // Method name (generates get_posts)
        table = "posts",        // Related table name
        remote_id = "author_id" // Foreign key in related table
    }, {
        entity = Comment,       // Multiple relationships allowed
        name = "comments",
        table = "comments",
        remote_id = "author_id"
    }]
)]
pub struct Author {
    #[georm(id)]
    pub id: i32,
    pub name: String,
}
```

**Generated methods**:
- `author.get_posts(pool).await? -> Vec<Post>`
- `author.get_comments(pool).await? -> Vec<Comment>`

##### Many-to-Many Relationships

For many-to-many relationships, specify the link table that connects the entities:

```sql
-- Example schema for books and genres
CREATE TABLE books (
    id SERIAL PRIMARY KEY,
    title VARCHAR(200) NOT NULL
);

CREATE TABLE genres (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL
);

CREATE TABLE book_genres (
    book_id INT NOT NULL REFERENCES books(id),
    genre_id INT NOT NULL REFERENCES genres(id),
    PRIMARY KEY (book_id, genre_id)
);
```

```rust
#[derive(Georm)]
#[georm(
    table = "books",
    many_to_many = [{
        entity = Genre,         // Related entity type
        name = "genres",        // Method name (generates get_genres)
        table = "genres",       // Related table name
        remote_id = "id",       // Primary key in related table (default: "id")
        link = {                // Link table configuration
            table = "book_genres",  // Join table name
            from = "book_id",       // Column referencing this entity
            to = "genre_id"         // Column referencing related entity
        }
    }]
)]
pub struct Book {
    #[georm(id)]
    pub id: i32,
    pub title: String,
}

#[derive(Georm)]
#[georm(
    table = "genres",
    many_to_many = [{
        entity = Book,
        name = "books",
        table = "books",
        link = {
            table = "book_genres",
            from = "genre_id",      // Note: reversed perspective
            to = "book_id"
        }
    }]
)]
pub struct Genre {
    #[georm(id)]
    pub id: i32,
    pub name: String,
}
```

**Generated methods**:
- `book.get_genres(pool).await? -> Vec<Genre>`
- `genre.get_books(pool).await? -> Vec<Book>`

#### Relationship Attribute Reference

| Attribute    | Description                                          | Required | Default |
|--------------|------------------------------------------------------|----------|---------|
| `entity`     | Target entity type                                   | Yes      | N/A     |
| `name`       | Method name (generates `get_{name}`)                 | Yes      | N/A     |
| `table`      | Target table name                                    | Yes      | N/A     |
| `remote_id`  | Target table's key column                            | No       | `"id"`  |
| `nullable`   | Whether relationship can be null (field-level only)  | No       | `false` |
| `link.table` | Join table name (many-to-many only)                  | Yes*     | N/A     |
| `link.from`  | Column referencing this entity (many-to-many only)   | Yes*     | N/A     |
| `link.to`    | Column referencing target entity (many-to-many only) | Yes*     | N/A     |

*Required for many-to-many relationships

#### Complex Relationship Example

Here's a comprehensive example showing multiple relationship types:

```rust
#[derive(Georm)]
#[georm(
    table = "posts",
    one_to_many = [{
        entity = Comment,
        name = "comments",
        table = "comments",
        remote_id = "post_id"
    }],
    many_to_many = [{
        entity = Tag,
        name = "tags",
        table = "tags",
        link = {
            table = "post_tags",
            from = "post_id",
            to = "tag_id"
        }
    }]
)]
pub struct Post {
    #[georm(id)]
    pub id: i32,
    pub title: String,
    pub content: String,

    // Field-level relationship (foreign key)
    #[georm(relation = {
        entity = Author,
        table = "authors",
        name = "author"
    })]
    pub author_id: i32,

    // Nullable field-level relationship
    #[georm(relation = {
        entity = Category,
        table = "categories",
        name = "category",
        nullable = true
    })]
    pub category_id: Option<i32>,
}
```

**Generated methods**:
- `post.get_author(pool).await? -> Author` (from field relation)
- `post.get_category(pool).await? -> Option<Category>` (nullable field relation)
- `post.get_comments(pool).await? -> Vec<Comment>` (one-to-many)
- `post.get_tags(pool).await? -> Vec<Tag>` (many-to-many)

## API Reference

### Core Operations

All entities implementing `Georm<Id>` get these methods:

```rust
// Query operations
Post::find_all(pool).await?;              // Find all posts
Post::find(pool, &post_id).await?;        // Find by ID

// Mutation operations
post.create(pool).await?;                 // Insert new record
post.update(pool).await?;                 // Update existing record
post.create_or_update(pool).await?;       // Upsert operation
post.delete(pool).await?;                 // Delete this record
Post::delete_by_id(pool, &post_id).await?; // Delete by ID

// Utility
post.get_id();                            // Get entity ID
```

### Defaultable Operations

Entities with defaultable fields get a companion `<Entity>Default` struct:

```rust
// Create with defaults
post_default.create(pool).await?;
```

## Configuration

### Attributes Reference

#### Struct-level attributes

```rust
#[georm(
    table = "table_name",                   // Required: database table name
    one_to_one = [{ /* ... */ }],           // Optional: one-to-one relationships
    one_to_many = [{ /* ... */ }],          // Optional: one-to-many relationships
    many_to_many = [{ /* ... */ }]          // Optional: many-to-many relationships
)]
```

#### Field-level attributes

```rust
#[georm(id)]                               // Mark as primary key
#[georm(defaultable)]                      // Mark as defaultable field
#[georm(relation = { /* ... */ })]         // Define relationship
```

## Performance

Georm is designed for zero runtime overhead:

- **Compile-time queries**: All SQL is verified at compile time
- **No reflection**: Direct field access, no runtime introspection
- **Minimal allocations**: Efficient use of owned vs borrowed data
- **SQLx integration**: Leverages SQLx's optimized PostgreSQL driver

## Examples

### Comprehensive Example

For an example showcasing user management, comments, and follower relationships, see the example in `examples/postgres/users-comments-and-followers/`. This example demonstrates:

- User management and profile management
- Comment system with user associations
- Follower/following relationships (many-to-many)
- Interactive CLI interface with CRUD operations
- Database migrations and schema setup

To run the example:

```bash
# Set up your database
export DATABASE_URL="postgres://username:password@localhost/georm_example"

# Run migrations
cargo sqlx migrate run

# Run the example
cd examples/postgres/users-comments-and-followers
cargo run help # For a list of all available actions
```

## Comparison

| Feature              | Georm | SeaORM | Diesel |
|----------------------|-------|--------|--------|
| Compile-time safety  | ✅   | ✅    | ✅    |
| Relationship support | ✅   | ✅    | ✅    |
| Async support        | ✅   | ✅    | ⚠️    |
| Learning curve       | Low   | Medium | High   |
| Macro simplicity     | ✅   | ❌    | ❌    |
| Advanced queries     | ❌   | ✅    | ✅    |

## Roadmap

### High Priority
- **Transaction Support**: Comprehensive transaction handling with atomic operations

### Medium Priority
- **Composite Key Relationships**: Add relationship support (one-to-one, one-to-many, many-to-many) for entities with composite primary keys
- **Multi-Database Support**: MySQL and SQLite support with feature flags
- **Field-Based Queries**: Generate `find_by_{field_name}` methods that return `Vec<T>` for regular fields or `Option<T>` for unique fields
- **Relationship Optimization**: Eager loading and N+1 query prevention
- **Soft Delete**: Optional soft delete with `deleted_at` timestamps

### Lower Priority
- **Migration Support**: Schema generation and evolution utilities
- **Enhanced Error Handling**: Custom error types with better context

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

#### Prerequisites

- **Rust 1.86+**: Georm uses modern Rust features and follows the MSRV specified in `rust-toolchain.toml`
- **PostgreSQL 12+**: Required for running tests and development
- **Git**: For version control
- **Jujutsu**: For version control (alternative to Git)

#### Required Tools

The following tools are used in the development workflow:

- **[just](https://github.com/casey/just)**: Task runner for common development commands
- **[cargo-deny](https://github.com/EmbarkStudios/cargo-deny)**: License and security auditing
- **[sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)**: Database migrations and management
- **[bacon](https://github.com/Canop/bacon)**: Background code checker (optional but recommended)

Install these tools:

```bash
# Install just (task runner)
cargo install just

# Install cargo-deny (for auditing)
cargo install cargo-deny

# Install sqlx-cli (for database management)
cargo install sqlx-cli --no-default-features --features native-tls,postgres

# Install bacon (optional, for live feedback)
cargo install bacon
```

#### Quick Start

```bash
# Clone the repository
git clone https://github.com/Phundrak/georm.git
cd georm

# Set up your PostgreSQL database and set DATABASE_URL
export DATABASE_URL="postgres://username:password@localhost/georm_test"

# Run migrations
just migrate

# Run all tests
just test

# Run linting
just lint

# Run security audit
just audit

# Run all checks (format, lint, audit, test)
just check-all
```

#### Available Commands (via just)

```bash
just                   # Default: run linting
just build             # Build the project
just build-release     # Build in release mode
just test              # Run all tests
just lint              # Run clippy linting
just audit             # Run security and license audit
just migrate           # Run database migrations
just format            # Format all code
just format-check      # Check code formatting
just check-all         # Run all checks (format, lint, audit, test)
just clean             # Clean build artifacts
```

#### Running Specific Tests

```bash
# Run tests for a specific module
cargo test --test simple_struct
cargo test --test defaultable_struct
cargo test --test m2m_relationship

# Run tests with output
cargo test -- --nocapture

# Run a specific test function
cargo test defaultable_struct_should_exist
```

#### Development with Bacon (Optional)

For continuous feedback during development:

```bash
# Run clippy continuously
bacon

# Run tests continuously
bacon test

# Build docs continuously
bacon doc
```

#### Devenv Development Environment (Optional)

If you use [Nix](https://nixos.org/), you can use the provided devenv configuration for a reproducible development environment:

```bash
# Enter the development shell with all tools pre-installed
devenv shell

# Or use direnv for automatic environment activation
direnv allow
```

The devenv configuration provides:
- Exact Rust version (1.86) with required components
- All development tools (just, cargo-deny, sqlx-cli, bacon)
- LSP support (rust-analyzer)
- SQL tooling (sqls for SQL language server)
- PostgreSQL database for development

**Devenv configuration:**
- **Rust toolchain**: Specified version with rustfmt, clippy, and rust-analyzer
- **Development tools**: just, cargo-deny, sqlx-cli, bacon
- **SQL tools**: sqls (SQL language server)
- **Database**: PostgreSQL with automatic setup
- **Platform support**: Cross-platform (Linux, macOS, etc.)

#### Database Setup for Tests

Tests require a PostgreSQL database. Set up a test database:

```sql
-- Connect to PostgreSQL as superuser
CREATE DATABASE georm_test;
CREATE USER georm_user WITH PASSWORD 'georm_password';
GRANT ALL PRIVILEGES ON DATABASE georm_test TO georm_user;
```

Set the environment variable:

```bash
export DATABASE_URL="postgres://georm_user:georm_password@localhost/georm_test"
```

#### IDE Setup

- Ensure `rust-analyzer` is configured
- Set up PostgreSQL connection for SQL syntax highlighting

#### Code Style

The project uses standard Rust formatting:

```bash
# Format code
just format

# Check formatting (CI)
just format-check
```

Clippy linting is enforced:

```bash
# Run linting
just lint

# Fix auto-fixable lints
cargo clippy --fix
```

## License

Licensed under either of

 * MIT License ([LICENSE-MIT](LICENSE-MIT.md) or http://opensource.org/licenses/MIT)
 * GNU General Public License v3.0 ([LICENSE-GPL](LICENSE-GPL.md) or https://www.gnu.org/licenses/gpl-3.0.html)

at your option.

## Acknowledgments

- Built on top of the excellent [SQLx](https://github.com/launchbadge/sqlx) library
- Inspired by [Hibernate](https://hibernate.org/)
