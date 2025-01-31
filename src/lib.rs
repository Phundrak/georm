pub use georm_macros::Georm;

pub trait Georm<Id> {
    /// Find all the entities in the database.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn find_all(
        pool: &sqlx::PgPool,
    ) -> impl ::std::future::Future<Output = ::sqlx::Result<Vec<Self>>> + Send
    where
        Self: Sized;

    /// Find the entiy in the database based on its identifier.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn find(
        pool: &sqlx::PgPool,
        id: &Id,
    ) -> impl std::future::Future<Output = sqlx::Result<Option<Self>>> + Send
    where
        Self: Sized;

    /// Create the entity in the database.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn create(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<Self>> + Send
    where
        Self: Sized;

    /// Update an entity with a matching identifier in the database.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn update(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<Self>> + Send
    where
        Self: Sized;

    /// Update an entity with a matching identifier in the database if
    /// it exists, create it otherwise.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn create_or_update(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl ::std::future::Future<Output = sqlx::Result<Self>>
    where
        Self: Sized,
    {
        async {
            if Self::find(pool, self.get_id()).await?.is_some() {
                self.update(pool).await
            } else {
                self.create(pool).await
            }
        }
    }

    /// Delete the entity from the database if it exists.
    ///
    /// # Returns
    /// Returns the amount of rows affected by the deletion.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn delete(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<u64>> + Send;

    /// Delete any entity with the identifier `id`.
    ///
    /// # Returns
    /// Returns the amount of rows affected by the deletion.
    ///
    /// # Errors
    /// Returns any error Postgres may have encountered
    fn delete_by_id(
        pool: &sqlx::PgPool,
        id: &Id,
    ) -> impl std::future::Future<Output = sqlx::Result<u64>> + Send;

    /// Returns the identifier of the entity.
    fn get_id(&self) -> &Id;
}
