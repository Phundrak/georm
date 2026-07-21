#![cfg(feature = "sqlite")]

use georm::Georm;

// Test struct with defaultable fields using existing table structure
#[derive(Georm, Debug)]
#[georm(table = "authors")]
struct TestAuthor {
    #[georm(id, defaultable)]
    pub id: i64,
    pub name: String,
    pub biography_id: Option<i64>, // Don't mark Option fields as defaultable
}

// Test struct with only ID defaultable
#[derive(Georm)]
#[georm(table = "authors")]
struct MinimalDefaultable {
    #[georm(id, defaultable)]
    pub id: i64,
    pub name: String,
    pub biography_id: Option<i64>,
}

// Test struct with multiple defaultable fields
#[derive(Georm)]
#[georm(table = "authors")]
struct MultiDefaultable {
    #[georm(id, defaultable)]
    pub id: i64,
    #[georm(defaultable)]
    pub name: String,
    pub biography_id: Option<i64>,
}

#[test]
fn defaultable_struct_should_exist() {
    let _author_default = TestAuthorDefault {
        id: Some(1),
        name: "Test Author".to_string(),
        biography_id: None,
    };
}

#[test]
fn minimal_defaultable_struct_should_exist() {
    let _minimal_default = MinimalDefaultableDefault {
        id: None,
        name: "testuser".to_string(),
        biography_id: None,
    };
}

#[test]
fn defaultable_fields_can_be_none() {
    let _author_default = TestAuthorDefault {
        id: None,
        name: "Test Author".to_string(),
        biography_id: None,
    };
}

#[test]
fn field_visibility_is_preserved() {
    let _author_default = TestAuthorDefault {
        id: Some(1),
        name: "Test".to_string(),
        biography_id: Some(1),
    };
}

mod defaultable_tests {
    use super::*;
    use georm::Defaultable;
    use sqlx::SqlitePool;

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_create_entity_from_defaultable_with_id(pool: SqlitePool) {
        let author_default = TestAuthorDefault {
            id: Some(999),
            name: "John Doe".to_string(),
            biography_id: None,
        };

        let created_author = author_default.create(&pool).await.unwrap();

        assert_eq!(created_author.id, 999);
        assert_eq!(created_author.name, "John Doe");
        assert_eq!(created_author.biography_id, None);
    }

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_create_entity_from_defaultable_without_id(pool: SqlitePool) {
        let author_default = TestAuthorDefault {
            id: None,
            name: "Jane Smith".to_string(),
            biography_id: None,
        };

        let created_author = author_default.create(&pool).await.unwrap();

        assert!(created_author.id > 0);
        assert_eq!(created_author.name, "Jane Smith");
        assert_eq!(created_author.biography_id, None);
    }

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_create_entity_from_minimal_defaultable(pool: SqlitePool) {
        let minimal_default = MinimalDefaultableDefault {
            id: None,
            name: "Alice Wonder".to_string(),
            biography_id: Some(1),
        };

        let created_author = minimal_default.create(&pool).await.unwrap();

        assert!(created_author.id > 0);
        assert_eq!(created_author.name, "Alice Wonder");
        assert_eq!(created_author.biography_id, Some(1));
    }

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_create_multiple_entities_from_defaultable(pool: SqlitePool) {
        let author1_default = TestAuthorDefault {
            id: None,
            name: "Author One".to_string(),
            biography_id: None,
        };

        let author2_default = TestAuthorDefault {
            id: None,
            name: "Author Two".to_string(),
            biography_id: None,
        };

        let created_author1 = author1_default.create(&pool).await.unwrap();
        let created_author2 = author2_default.create(&pool).await.unwrap();

        assert!(created_author1.id > 0);
        assert!(created_author2.id > 0);
        assert_ne!(created_author1.id, created_author2.id);

        assert_eq!(created_author1.name, "Author One");
        assert_eq!(created_author2.name, "Author Two");
    }

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_multiple_defaultable_fields_mixed(pool: SqlitePool) {
        let multi_default = MultiDefaultableDefault {
            id: None,
            name: Some("Explicit Name".to_string()),
            biography_id: Some(1),
        };

        let created = multi_default.create(&pool).await.unwrap();

        assert!(created.id > 0);
        assert_eq!(created.name, "Explicit Name");
        assert_eq!(created.biography_id, Some(1));
    }

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_multiple_defaultable_fields_all_explicit(pool: SqlitePool) {
        let multi_default = MultiDefaultableDefault {
            id: Some(888),
            name: Some("All Explicit".to_string()),
            biography_id: None,
        };

        let created = multi_default.create(&pool).await.unwrap();

        assert_eq!(created.id, 888);
        assert_eq!(created.name, "All Explicit");
        assert_eq!(created.biography_id, None);
    }

    #[sqlx::test(
        migrations = "./migrations/sqlite",
        fixtures(path = "fixtures", scripts("simple_struct"))
    )]
    async fn test_error_duplicate_id(pool: SqlitePool) {
        let author1 = TestAuthorDefault {
            id: Some(777),
            name: "First Author".to_string(),
            biography_id: None,
        };

        let author2 = TestAuthorDefault {
            id: Some(777),
            name: "Second Author".to_string(),
            biography_id: None,
        };

        let _created1 = author1.create(&pool).await.unwrap();

        let result2 = author2.create(&pool).await;
        assert!(result2.is_err());
    }

    mod sql_validation_tests {
        use super::*;

        #[sqlx::test(
            migrations = "./migrations/sqlite",
            fixtures(path = "fixtures", scripts("simple_struct"))
        )]
        async fn test_sql_generation_no_defaultable_fields(pool: SqlitePool) {
            let author_default = TestAuthorDefault {
                id: Some(100),
                name: "Test Name".to_string(),
                biography_id: Some(1),
            };

            let created = author_default.create(&pool).await.unwrap();

            assert_eq!(created.id, 100);
            assert_eq!(created.name, "Test Name");
            assert_eq!(created.biography_id, Some(1));

            let found: TestAuthor = sqlx::query_as!(
                TestAuthor,
                "SELECT id, name, biography_id FROM authors WHERE id = ?",
                100
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(found.id, 100);
            assert_eq!(found.name, "Test Name");
            assert_eq!(found.biography_id, Some(1));
        }

        #[sqlx::test(
            migrations = "./migrations/sqlite",
            fixtures(path = "fixtures", scripts("simple_struct"))
        )]
        async fn test_sql_generation_with_defaultable_none(pool: SqlitePool) {
            let author_default = TestAuthorDefault {
                id: None,
                name: "Auto ID Test".to_string(),
                biography_id: None,
            };

            let created = author_default.create(&pool).await.unwrap();

            assert!(created.id > 0);
            assert_eq!(created.name, "Auto ID Test");
            assert_eq!(created.biography_id, None);
        }

        #[sqlx::test(
            migrations = "./migrations/sqlite",
            fixtures(path = "fixtures", scripts("simple_struct"))
        )]
        async fn test_placeholder_ordering_consistency(pool: SqlitePool) {
            let record2 = MultiDefaultableDefault {
                id: Some(201),
                name: Some("Full Record".to_string()),
                biography_id: Some(1),
            };

            let record3 = MultiDefaultableDefault {
                id: None,
                name: Some("Mixed Record".to_string()),
                biography_id: None,
            };

            let created2 = record2.create(&pool).await.unwrap();
            assert_eq!(created2.id, 201);
            assert_eq!(created2.name, "Full Record");
            assert_eq!(created2.biography_id, Some(1));

            let created3 = record3.create(&pool).await.unwrap();
            assert!(created3.id > 0);
            assert_eq!(created3.name, "Mixed Record");
            assert_eq!(created3.biography_id, None);
        }

        #[sqlx::test(
            migrations = "./migrations/sqlite",
            fixtures(path = "fixtures", scripts("simple_struct"))
        )]
        async fn test_returning_clause_functionality(pool: SqlitePool) {
            let author_default = TestAuthorDefault {
                id: None,
                name: "Return Test".to_string(),
                biography_id: None,
            };

            let created = author_default.create(&pool).await.unwrap();

            assert!(created.id > 0);
            assert_eq!(created.name, "Return Test");
            assert_eq!(created.biography_id, None);

            let verified: TestAuthor = sqlx::query_as!(
                TestAuthor,
                "SELECT id, name, biography_id FROM authors WHERE id = ?",
                created.id
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(verified.id, created.id);
            assert_eq!(verified.name, created.name);
            assert_eq!(verified.biography_id, created.biography_id);
        }
    }
}
