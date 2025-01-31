use georm::Georm;

mod models;
use models::*;

#[sqlx::test(fixtures("simple_struct", "o2o"))]
async fn book_should_have_working_get_author_method(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let book = Book::find(&pool, &1).await?;
    assert!(book.is_some());
    let book = book.unwrap();
    let author = book.get_author(&pool).await?;
    let expected_author = Author {
        id: 1,
        name: "J.R.R. Tolkien".into(),
        biography_id: Some(2),
    };
    assert_eq!(expected_author, author);
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn author_should_have_working_get_biography_method(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let author = Author::find(&pool, &1).await?;
    assert!(author.is_some());
    let author = author.unwrap();
    let biography = author.get_biography(&pool).await?;
    assert!(biography.is_some());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn author_should_have_optional_biographies(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let tolkien = Author::find(&pool, &1).await?;
    assert!(tolkien.is_some());
    let tolkien_biography = tolkien.unwrap().get_biography(&pool).await?;
    assert!(tolkien_biography.is_some());
    let biography = Biography {
        id: 2,
        content: "Some other text".into(),
    };
    assert_eq!(biography, tolkien_biography.unwrap());
    let orwell = Author::find(&pool, &2).await?;
    assert!(orwell.is_some());
    assert!(orwell.unwrap().get_biography(&pool).await?.is_none());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct", "o2o"))]
async fn books_are_found_despite_nonstandard_id_name(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let review = Review::find(&pool, &1).await?.unwrap();
    let book = review.get_book(&pool).await?;
    let tolkien = Author::find(&pool, &1).await?.unwrap();
    assert_eq!(tolkien, book.get_author(&pool).await?);
    Ok(())
}
