use georm::Georm;
use rand::seq::SliceRandom;

use models::Author;
mod models;

#[sqlx::test(fixtures("simple_struct"))]
async fn find_all_query_works(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let result = Author::find_all(&pool).await?;
    assert_eq!(3, result.len());
    Ok(())
}

#[sqlx::test]
async fn find_all_returns_empty_vec_on_empty_table(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let result = Author::find_all(&pool).await?;
    assert_eq!(0, result.len());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn find_query_works(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let id = 1;
    let res = Author::find(&pool, &id).await?;
    assert!(res.is_some());
    let res = res.unwrap();
    assert_eq!(String::from("J.R.R. Tolkien"), res.name);
    assert_eq!(1, res.id);
    Ok(())
}

#[sqlx::test]
async fn find_returns_none_if_not_found(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let res = Author::find(&pool, &420).await?;
    assert!(res.is_none());
    Ok(())
}

#[sqlx::test]
async fn create_works(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let author = Author {
        id: 1,
        name: "J.R.R. Tolkien".into(),
        ..Default::default()
    };
    author.create(&pool).await?;
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(1, all_authors.len());
    assert_eq!(vec![author], all_authors);
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn create_fails_if_already_exists(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let author = Author {
        id: 2,
        name: "Miura Kentaro".into(),
        ..Default::default()
    };
    let result = author.create(&pool).await;
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert_eq!("error returned from database: duplicate key value violates unique constraint \"authors_pkey\"", error.to_string());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn update_works(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let expected_initial = Author {
        name: "J.R.R. Tolkien".into(),
        id: 1,
        biography_id: Some(2),
    };
    let expected_final = Author {
        name: "Jolkien Rolkien Rolkien Tolkien".into(),
        id: 1,
        biography_id: Some(2),
    };
    let tolkien = Author::find(&pool, &1).await?;
    assert!(tolkien.is_some());
    let mut tolkien = tolkien.unwrap();
    assert_eq!(expected_initial, tolkien);
    tolkien.name = expected_final.name.clone();
    let updated = tolkien.update(&pool).await?;
    assert_eq!(expected_final, updated);
    Ok(())
}

#[sqlx::test]
async fn update_fails_if_not_already_exists(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let author = Author {
        id: 2,
        name: "Miura Kentaro".into(),
        ..Default::default()
    };
    let result = author.update(&pool).await;
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert_eq!(
        "no rows returned by a query that expected to return at least one row",
        error.to_string()
    );
    Ok(())
}

#[sqlx::test]
async fn should_create_if_does_not_exist(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(0, all_authors.len());
    let author = Author {
        id: 4,
        name: "Miura Kentaro".into(),
        ..Default::default()
    };
    author.create_or_update(&pool).await?;
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(1, all_authors.len());
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn should_update_if_exist(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(3, all_authors.len());
    let author = Author {
        id: 2,
        name: "Miura Kentaro".into(),
        ..Default::default()
    };
    author.create_or_update(&pool).await?;
    let mut all_authors = Author::find_all(&pool).await?;
    all_authors.sort();
    assert_eq!(3, all_authors.len());
    assert_eq!(author, all_authors[1]);
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn delete_by_id_should_delete_only_one_entry(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let id = 2;
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(3, all_authors.len());
    assert!(all_authors.iter().any(|author| author.get_id() == &id));
    let result = Author::delete_by_id(&pool, &id).await?;
    assert_eq!(1, result);
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(2, all_authors.len());
    assert!(all_authors.iter().all(|author| author.get_id() != &id));
    Ok(())
}

#[sqlx::test(fixtures("simple_struct"))]
async fn delete_should_delete_current_entity_from_db(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let mut all_authors = Author::find_all(&pool).await?;
    assert_eq!(3, all_authors.len());
    all_authors.shuffle(&mut rand::rng());
    let author = all_authors.first().unwrap();
    let result = author.delete(&pool).await?;
    assert_eq!(1, result);
    let all_authors = Author::find_all(&pool).await?;
    assert_eq!(2, all_authors.len());
    assert!(all_authors.iter().all(|a| a.get_id() != author.get_id()));
    Ok(())
}
