pub trait Defaultable<Id, Entity> {
    /// Creates an entity in the database.
    ///
    /// # Errors
    /// Returns any error the database may have encountered
    fn create(
        &self,
        pool: &sqlx::PgPool,
    ) -> impl std::future::Future<Output = sqlx::Result<Entity>> + Send;
}
