/// Core database operations trait for Georm entities.
///
/// This trait is automatically implemented by the `#[derive(Georm)]` macro and provides
/// all essential CRUD operations for database entities. The trait is generic over the
/// primary key type `Id`, which can be a simple type (e.g., `i32`) or a generated
/// composite key struct (e.g., `UserRoleId`).
///
/// ## Generated Implementation
///
/// When you derive `Georm` on a struct, this trait is automatically implemented with
/// PostgreSQL-optimized queries that use:
/// - **Prepared statements** for security and performance
/// - **RETURNING clause** to capture database-generated values
/// - **ON CONFLICT** for efficient upsert operations
/// - **Compile-time verification** via SQLx macros
///
/// ## Method Categories
///
/// ### Static Methods (Query Operations)
/// - [`find_all`] - Retrieve all entities from the table
/// - [`find`] - Retrieve a single entity by primary key
/// - [`delete_by_id`] - Delete an entity by primary key
///
/// ### Instance Methods (Mutation Operations)
/// - [`create`] - Insert a new entity into the database
/// - [`update`] - Update an existing entity in the database
/// - [`create_or_update`] - Upsert (insert or update) an entity
/// - [`delete`] - Delete this entity from the database
/// - [`get_id`] - Get the primary key of this entity
///
/// ## Usage Examples
///
/// ```ignore
/// use georm::Georm;
///
/// #[derive(Georm)]
/// #[georm(table = "users")]
/// struct User {
///     #[georm(id)]
///     id: i32,
///     username: String,
///     email: String,
/// }
///
/// // Static methods
/// let all_users = User::find_all(&pool).await?;
/// let user = User::find(&pool, &1).await?;
/// let deleted_count = User::delete_by_id(&pool, &1).await?;
///
/// // Instance methods
/// let new_user = User { id: 0, username: "alice".into(), email: "alice@example.com".into() };
/// let created = new_user.create(&pool).await?;
/// let updated = created.update(&pool).await?;
/// let id = updated.get_id();
/// let deleted_count = updated.delete(&pool).await?;
/// ```
///
/// ## Composite Key Support
///
/// For entities with composite primary keys, the `Id` type parameter becomes a generated
/// struct following the pattern `{EntityName}Id`:
///
/// ```ignore
/// #[derive(Georm)]
/// #[georm(table = "user_roles")]
/// struct UserRole {
///     #[georm(id)]
///     user_id: i32,
///     #[georm(id)]
///     role_id: i32,
///     assigned_at: chrono::DateTime<chrono::Utc>,
/// }
///
/// // Generated: pub struct UserRoleId { pub user_id: i32, pub role_id: i32 }
/// // Trait: impl Georm<UserRoleId> for UserRole
///
/// let id = UserRoleId { user_id: 1, role_id: 2 };
/// let user_role = UserRole::find(&pool, &id).await?;
/// ```
///
/// ## Error Handling
///
/// All methods return `sqlx::Result<T>` and may fail due to:
/// - Database connection issues
/// - Constraint violations (unique, foreign key, etc.)
/// - Invalid queries (though most are caught at compile time)
/// - Missing records (for operations expecting existing data)
///
/// [`find_all`]: Georm::find_all
/// [`find`]: Georm::find
/// [`create`]: Georm::create
/// [`update`]: Georm::update
/// [`create_or_update`]: Georm::create_or_update
/// [`delete`]: Georm::delete
/// [`delete_by_id`]: Georm::delete_by_id
/// [`get_id`]: Georm::get_id
pub trait Georm<Id> {
    /// Retrieve all entities from the database table.
    ///
    /// This method executes a `SELECT * FROM table_name` query and returns all records
    /// as a vector of entities. The results are not paginated or filtered.
    ///
    /// # Returns
    /// - `Ok(Vec<Self>)` - All entities in the table (may be empty)
    /// - `Err(sqlx::Error)` - Database connection or query execution errors
    ///
    /// # Performance Notes
    /// - Returns all records in memory - consider pagination for large tables
    /// - Uses prepared statements for optimal performance
    /// - No built-in ordering - results may vary between calls
    ///
    /// # Examples
    /// ```ignore
    /// let all_users = User::find_all(&pool).await?;
    /// println!("Found {} users", all_users.len());
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for database connection issues, permission problems,
    /// or if the table doesn't exist.
    fn find_all(
        pool: &sqlx::PgPool,
    ) -> impl ::std::future::Future<Output = ::sqlx::Result<Vec<Self>>> + Send
    where
        Self: Sized;

    /// Find a single entity by its primary key.
    ///
    /// This method executes a `SELECT * FROM table_name WHERE primary_key = $1` query
    /// (or equivalent for composite keys) and returns the matching entity if found.
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    /// - `id` - Primary key value (simple type or composite key struct)
    ///
    /// # Returns
    /// - `Ok(Some(Self))` - Entity found and returned
    /// - `Ok(None)` - No entity with the given ID exists
    /// - `Err(sqlx::Error)` - Database connection or query execution errors
    ///
    /// # Examples
    /// ```ignore
    /// // Simple primary key
    /// let user = User::find(&pool, &1).await?;
    ///
    /// // Composite primary key
    /// let id = UserRoleId { user_id: 1, role_id: 2 };
    /// let user_role = UserRole::find(&pool, &id).await?;
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for database connection issues, type conversion errors,
    /// or query execution problems. Note that not finding a record is not an error
    /// - it returns `Ok(None)`.
    fn find(
        pool: &sqlx::PgPool,
        id: &Id,
    ) -> impl std::future::Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        Self: Sized;

    /// Insert this entity as a new record in the database.
    ///
    /// This method executes an `INSERT INTO table_name (...) VALUES (...) RETURNING *`
    /// query and returns the newly created entity with any database-generated values
    /// (such as auto-increment IDs, default timestamps, etc.).
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    ///
    /// # Returns
    /// - `Ok(Self)` - The entity as it exists in the database after insertion
    /// - `Err(sqlx::Error)` - Database constraint violations or connection errors
    ///
    /// # Database Behavior
    /// - Uses `RETURNING *` to capture database-generated values
    /// - Respects database defaults for fields marked `#[georm(defaultable)]`
    /// - Triggers and database-side modifications are reflected in the returned entity
    ///
    /// # Examples
    /// ```ignore
    /// let new_user = User { id: 0, username: "alice".into(), email: "alice@example.com".into() };
    /// let created_user = new_user.create(&pool).await?;
    /// println!("Created user with ID: {}", created_user.id);
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for:
    /// - Unique constraint violations
    /// - Foreign key constraint violations  
    /// - NOT NULL constraint violations
    /// - Database connection issues
    /// - Permission problems
    fn create(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<Self>> + Send
    where
        Self: Sized;

    /// Update an existing entity in the database.
    ///
    /// This method executes an `UPDATE table_name SET ... WHERE primary_key = ... RETURNING *`
    /// query using the entity's current primary key to locate the record to update.
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    ///
    /// # Returns
    /// - `Ok(Self)` - The entity as it exists in the database after the update
    /// - `Err(sqlx::Error)` - Database errors or if no matching record exists
    ///
    /// # Database Behavior
    /// - Uses `RETURNING *` to capture any database-side changes
    /// - Updates all fields, not just changed ones
    /// - Triggers and database-side modifications are reflected in the returned entity
    /// - Fails if no record with the current primary key exists
    ///
    /// # Examples
    /// ```ignore
    /// let mut user = User::find(&pool, &1).await?.unwrap();
    /// user.email = "newemail@example.com".into();
    /// let updated_user = user.update(&pool).await?;
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for:
    /// - No matching record found (record was deleted by another process)
    /// - Constraint violations (unique, foreign key, etc.)
    /// - Database connection issues
    /// - Permission problems
    fn update(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<Self>> + Send
    where
        Self: Sized;

    /// Insert or update this entity using PostgreSQL's upsert functionality.
    ///
    /// This method executes an `INSERT ... ON CONFLICT (...) DO UPDATE SET ... RETURNING *`
    /// query that atomically inserts the entity if it doesn't exist, or updates it if
    /// a record with the same primary key already exists.
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    ///
    /// # Returns
    /// - `Ok(Self)` - The final entity state in the database (inserted or updated)
    /// - `Err(sqlx::Error)` - Database connection or constraint violation errors
    ///
    /// # Database Behavior
    /// - Uses PostgreSQL's `ON CONFLICT` for true atomic upsert
    /// - More efficient than separate find-then-create-or-update logic
    /// - Uses `RETURNING *` to capture the final state
    /// - Conflict resolution is based on the primary key constraint
    ///
    /// # Examples
    /// ```ignore
    /// let user = User { id: 1, username: "alice".into(), email: "alice@example.com".into() };
    /// let final_user = user.create_or_update(&pool).await?;
    /// // Will insert if ID 1 doesn't exist, update if it does
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for:
    /// - Non-primary-key constraint violations
    /// - Database connection issues
    /// - Permission problems
    fn create_or_update(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl ::std::future::Future<Output = sqlx::Result<Self>>
    where
        Self: Sized;

    /// Delete this entity from the database.
    ///
    /// This method executes a `DELETE FROM table_name WHERE primary_key = ...` query
    /// using this entity's primary key to identify the record to delete.
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    ///
    /// # Returns
    /// - `Ok(u64)` - Number of rows affected (0 if entity didn't exist, 1 if deleted)
    /// - `Err(sqlx::Error)` - Database connection or constraint violation errors
    ///
    /// # Database Behavior
    /// - Uses the entity's current primary key for deletion
    /// - Returns 0 if no matching record exists (not an error)
    /// - May fail due to foreign key constraints if other records reference this entity
    ///
    /// # Examples
    /// ```ignore
    /// let user = User::find(&pool, &1).await?.unwrap();
    /// let deleted_count = user.delete(&pool).await?;
    /// assert_eq!(deleted_count, 1);
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for:
    /// - Foreign key constraint violations (referenced by other tables)
    /// - Database connection issues
    /// - Permission problems
    fn delete(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<u64>> + Send;

    /// Delete an entity by its primary key without needing an entity instance.
    ///
    /// This method executes a `DELETE FROM table_name WHERE primary_key = ...` query
    /// using the provided ID to identify the record to delete.
    ///
    /// # Parameters
    /// - `pool` - Database connection pool
    /// - `id` - Primary key value (simple type or composite key struct)
    ///
    /// # Returns
    /// - `Ok(u64)` - Number of rows affected (0 if entity didn't exist, 1 if deleted)
    /// - `Err(sqlx::Error)` - Database connection or constraint violation errors
    ///
    /// # Database Behavior
    /// - More efficient than `find().delete()` when you only have the ID
    /// - Returns 0 if no matching record exists (not an error)
    /// - May fail due to foreign key constraints if other records reference this entity
    ///
    /// # Examples
    /// ```ignore
    /// // Simple primary key
    /// let deleted_count = User::delete_by_id(&pool, &1).await?;
    ///
    /// // Composite primary key
    /// let id = UserRoleId { user_id: 1, role_id: 2 };
    /// let deleted_count = UserRole::delete_by_id(&pool, &id).await?;
    /// ```
    ///
    /// # Errors
    /// Returns `sqlx::Error` for:
    /// - Foreign key constraint violations (referenced by other tables)
    /// - Database connection issues
    /// - Permission problems
    fn delete_by_id(
        pool: &sqlx::PgPool,
        id: &Id,
    ) -> impl std::future::Future<Output = sqlx::Result<u64>> + Send;

    /// Get the primary key of this entity.
    ///
    /// For entities with simple primary keys, this returns the ID value directly.
    /// For entities with composite primary keys, this returns an owned instance of
    /// the generated `{EntityName}Id` struct.
    ///
    /// # Returns
    /// - Simple keys: The primary key value (e.g., `i32`, `String`)
    /// - Composite keys: Generated ID struct (e.g., `UserRoleId`)
    ///
    /// # Examples
    /// ```ignore
    /// // Simple primary key
    /// let user = User { id: 42, username: "alice".into(), email: "alice@example.com".into() };
    /// let id = user.get_id(); // Returns 42
    ///
    /// // Composite primary key  
    /// let user_role = UserRole { user_id: 1, role_id: 2, assigned_at: now };
    /// let id = user_role.get_id(); // Returns UserRoleId { user_id: 1, role_id: 2 }
    /// ```
    fn get_id(&self) -> Id;
}
