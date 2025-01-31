use georm::Georm;

mod models;
use models::*;

#[sqlx::test(fixtures("simple_struct", "o2o", "m2m"))]
async fn genres_should_be_able_to_access_all_books(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let fantasy = Genre::find(&pool, &1).await?.unwrap();
    let books = fantasy.get_books(&pool).await?;
    assert_eq!(3, books.len());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct", "o2o", "m2m"))]
async fn books_should_be_able_to_access_their_genres(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let to_build_a_fire = Book::find(&pool, &4).await?.unwrap();
    let genres = to_build_a_fire.get_genres(&pool).await?;
    assert_eq!(2, genres.len());
    Ok(())
}
