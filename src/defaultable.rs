/// Trait for creating entities with database defaults and auto-generated values.
///
/// This trait is automatically implemented on generated companion structs for entities
/// that have fields marked with `#[georm(defaultable)]`. It provides a convenient way
/// to create entities while allowing the database to provide default values for certain
/// fields.
///
/// ## Generated Implementation
///
/// When you mark fields with `#[georm(defaultable)]`, Georm automatically generates:
/// - A companion struct named `{EntityName}Default`
/// - An implementation of this trait for the companion struct
/// - Optimized SQL that omits defaultable fields when they are `None`
///
/// ## How It Works
///
/// The generated companion struct transforms defaultable fields into `Option<T>` types:
/// - `Some(value)` - Use the provided value
/// - `None` - Let the database provide the default value
///
/// Non-defaultable fields remain unchanged and are always required.
///
/// ## Database Behavior
///
/// The `create` method generates SQL that:
/// - Only includes fields where `Some(value)` is provided
/// - Omits fields that are `None`, allowing database defaults to apply
/// - Uses `RETURNING *` to capture the final entity state with all defaults applied
/// - Respects database triggers, sequences, and default value expressions
///
/// ## Usage Examples
///
/// ```ignore
/// use georm::{Georm, Defaultable};
///
/// #[derive(Georm)]
/// #[georm(table = "posts")]
/// pub struct Post {
///     #[georm(id, defaultable)]
///     id: i32,                    // Auto-generated serial
///     title: String,              // Required field
///     #[georm(defaultable)]
///     published: bool,            // Database default: false
///     #[georm(defaultable)]
///     created_at: chrono::DateTime<chrono::Utc>, // Database default: NOW()
///     author_id: i32,             // Required field
/// }
///
/// // Generated automatically:
/// // pub struct PostDefault {
/// //     pub id: Option<i32>,
/// //     pub title: String,
/// //     pub published: Option<bool>,
/// //     pub created_at: Option<chrono::DateTime<chrono::Utc>>,
/// //     pub author_id: i32,
/// // }
/// //
/// // impl Defaultable<i32, Post> for PostDefault { ... }
///
/// // Create with some defaults
/// let post_default = PostDefault {
///     id: None,                   // Let database auto-generate
///     title: "My Blog Post".to_string(),
///     published: None,            // Use database default (false)
///     created_at: None,           // Use database default (NOW())
///     author_id: 42,
/// };
///
/// let created_post = post_default.create(&pool).await?;
/// println!("Created post with ID: {}", created_post.id);
///
/// // Create with explicit values
/// let post_default = PostDefault {
///     id: None,                   // Still auto-generate ID
///     title: "Published Post".to_string(),
///     published: Some(true),      // Override default
///     created_at: Some(specific_time),  // Override default
///     author_id: 42,
/// };
/// ```
///
/// ## Type Parameters
///
/// - `Id` - The primary key type of the target entity (e.g., `i32`, `UserRoleId`)
/// - `Entity` - The target entity type that will be created (e.g., `Post`, `User`)
///
/// ## Comparison with Regular Creation
///
/// ```ignore
/// // Using regular Georm::create - must provide all values
/// let post = Post {
///     id: 0,                      // Ignored for auto-increment, but required
///     title: "My Post".to_string(),
///     published: false,           // Must specify even if it's the default
///     created_at: chrono::Utc::now(), // Must calculate current time manually
///     author_id: 42,
/// };
/// let created = post.create(&pool).await?;
///
/// // Using Defaultable::create - let database handle defaults
/// let post_default = PostDefault {
///     id: None,                   // Clearer intent for auto-generation
///     title: "My Post".to_string(),
///     published: None,            // Let database default apply
///     created_at: None,           // Let database calculate NOW()
///     author_id: 42,
/// };
/// let created = post_default.create(&pool).await?;
/// ```
///
/// ## Field Visibility
///
/// The generated companion struct preserves the field visibility of the original entity:
///
/// ```ignore
/// #[derive(Georm)]
/// #[georm(table = "posts")]
/// pub struct Post {
///     #[georm(id, defaultable)]
///     pub id: i32,
///     pub title: String,
///     #[georm(defaultable)]
///     pub(crate) internal_status: String,  // Crate-private field
///     #[georm(defaultable)]
///     private_field: String,               // Private field
/// }
///
/// // Generated with preserved visibility:
/// // pub struct PostDefault {
/// //     pub id: Option<i32>,
/// //     pub title: String,
/// //     pub(crate) internal_status: Option<String>,  // Preserved
/// //     private_field: Option<String>,               // Preserved
/// // }
/// ```
///
/// ## Limitations and Rules
///
/// - **Option fields cannot be defaultable**: Fields that are already `Option<T>` cannot
///   be marked with `#[georm(defaultable)]` to prevent `Option<Option<T>>` types
/// - **Compile-time validation**: Attempts to mark `Option<T>` fields as defaultable
///   result in compile-time errors
/// - **Requires at least one defaultable field**: The companion struct is only generated
///   if at least one field is marked as defaultable
/// - **No partial updates**: This trait only supports creating new entities, not updating
///   existing ones with defaults
///
/// ## Error Handling
///
/// The `create` method can fail for the same reasons as regular entity creation:
/// - Database connection issues
/// - Constraint violations (unique, foreign key, NOT NULL for non-defaultable fields)
/// - Permission problems
/// - Table or column doesn't exist
///
/// ## Performance Characteristics
///
/// - **Efficient SQL**: Only includes necessary fields in the INSERT statement
/// - **Single round-trip**: Uses `RETURNING *` to get the final entity state
/// - **No overhead**: Defaultable logic is resolved at compile time
/// - **Database-optimized**: Leverages database defaults rather than application logic
pub trait Defaultable<Id, Entity> {
    /// Create a new entity in the database using database defaults for unspecified fields.
    ///
    /// This method constructs and executes an `INSERT INTO table_name (...) VALUES (...) RETURNING *`
    /// query that only includes fields where `Some(value)` is provided. Fields that are `None`
    /// are omitted from the query, allowing the database to apply default values, auto-increment
    /// sequences, or trigger-generated values.
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    ///
    /// # Returns
    /// - `Ok(Entity)` - The newly created entity with all database-generated values populated
    /// - `Err(sqlx::Error)` - Database constraint violations or connection errors
    ///
    /// # Database Behavior
    /// - **Selective field inclusion**: Only includes fields with `Some(value)` in the INSERT
    /// - **Default value application**: Database defaults apply to omitted fields
    /// - **RETURNING clause**: Captures the complete entity state after insertion
    /// - **Trigger execution**: Database triggers run and their effects are captured
    /// - **Sequence generation**: Auto-increment values are generated and returned
    ///
    /// # SQL Generation
    ///
    /// The generated SQL dynamically includes only the necessary fields:
    ///
    /// ```sql
    /// -- If id=None, published=None, created_at=None:
    /// INSERT INTO posts (title, author_id) VALUES ($1, $2) RETURNING *;
    ///
    /// -- If id=None, published=Some(true), created_at=None:
    /// INSERT INTO posts (title, published, author_id) VALUES ($1, $2, $3) RETURNING *;
    /// ```
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Minimal creation - let database handle all defaults
    /// let post_default = PostDefault {
    ///     id: None,                   // Auto-generated
    ///     title: "Hello World".to_string(),
    ///     published: None,            // Database default
    ///     created_at: None,           // Database default (NOW())
    ///     author_id: 1,
    /// };
    /// let post = post_default.create(&pool).await?;
    ///
    /// // Mixed creation - some explicit values, some defaults
    /// let post_default = PostDefault {
    ///     id: None,                   // Auto-generated
    ///     title: "Published Post".to_string(),
    ///     published: Some(true),      // Override default
    ///     created_at: None,           // Still use database default
    ///     author_id: 1,
    /// };
    /// let post = post_default.create(&pool).await?;
    ///
    /// // Full control - specify all defaultable values
    /// let specific_time = chrono::Utc::now() - chrono::Duration::hours(1);
    /// let post_default = PostDefault {
    ///     id: Some(100),              // Explicit ID (if not auto-increment)
    ///     title: "Backdated Post".to_string(),
    ///     published: Some(false),     // Explicit value
    ///     created_at: Some(specific_time), // Explicit timestamp
    ///     author_id: 1,
    /// };
    /// let post = post_default.create(&pool).await?;
    /// ```
    ///
    /// # Error Conditions
    ///
    /// Returns `sqlx::Error` for:
    /// - **Unique constraint violations**: Duplicate values for unique fields
    /// - **Foreign key violations**: Invalid references to other tables
    /// - **NOT NULL violations**: Missing values for required non-defaultable fields
    /// - **Check constraint violations**: Values that don't meet database constraints
    /// - **Database connection issues**: Network or connection pool problems
    /// - **Permission problems**: Insufficient privileges for the operation
    /// - **Table/column errors**: Missing tables or columns (usually caught at compile time)
    ///
    /// # Performance Notes
    ///
    /// - **Optimal field selection**: Only transmits necessary data to the database
    /// - **Single database round-trip**: INSERT and retrieval in one operation
    /// - **Compile-time optimization**: Field inclusion logic resolved at compile time
    /// - **Database-native defaults**: Leverages database performance for default value generation
    ///
    /// # Comparison with Standard Creation
    ///
    /// ```ignore
    /// // Standard Georm::create - all fields required
    /// let post = Post {
    ///     id: 0,                      // Placeholder for auto-increment
    ///     title: "My Post".to_string(),
    ///     published: false,           // Must specify, even if it's the default
    ///     created_at: chrono::Utc::now(), // Must calculate manually
    ///     author_id: 1,
    /// };
    /// let created = post.create(&pool).await?;
    ///
    /// // Defaultable::create - only specify what you need
    /// let post_default = PostDefault {
    ///     id: None,                   // Clear intent for auto-generation
    ///     title: "My Post".to_string(),
    ///     published: None,            // Let database decide
    ///     created_at: None,           // Let database calculate
    ///     author_id: 1,
    /// };
    /// let created = post_default.create(&pool).await?;
    /// ```
    fn create(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<Entity>> + Send
    where
        Self: Sized;
}
