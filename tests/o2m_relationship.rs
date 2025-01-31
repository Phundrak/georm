use georm::Georm;

mod models;
use models::*;

#[sqlx::test(fixtures("simple_struct", "o2o"))]
async fn books_access_one_review(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let book = Book::find(&pool, &1).await?.unwrap();
    let reviews = book.get_reviews(&pool).await?;
    let review = Review {
        id: 1,
        book_id: 1,
        review: "Great book".into(),
    };
    assert_eq!(vec![review], reviews);
    Ok(())
}

#[sqlx::test(fixtures("simple_struct", "o2o"))]
async fn books_should_access_their_multiple_reviews(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let book = Book::find(&pool, &2).await?.unwrap();
    let reviews = book.get_reviews(&pool).await?;
    assert_eq!(2, reviews.len());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct", "o2o"))]
async fn books_can_have_no_reviews(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let book = Book::find(&pool, &4).await?.unwrap();
    let reviews = book.get_reviews(&pool).await?;
    assert_eq!(0, reviews.len());
    Ok(())
}
