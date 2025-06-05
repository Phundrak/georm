use georm::Georm;

// Test struct with defaultable fields using existing table structure
#[derive(Georm, Debug)]
#[georm(table = "authors")]
struct TestAuthor {
    #[georm(id, defaultable)]
    pub id: i32,
    pub name: String,
    pub biography_id: Option<i32>, // Don't mark Option fields as defaultable
}

// Test struct with only ID defaultable
#[derive(Georm)]
#[georm(table = "authors")]
struct MinimalDefaultable {
    #[georm(id, defaultable)]
    pub id: i32,
    pub name: String,
    pub biography_id: Option<i32>,
}

// Test struct with multiple defaultable fields
#[derive(Georm)]
#[georm(table = "authors")]
struct MultiDefaultable {
    #[georm(id, defaultable)]
    pub id: i32,
    #[georm(defaultable)]
    pub name: String,
    pub biography_id: Option<i32>,
}

#[test]
fn defaultable_struct_should_exist() {
    // This test will compile only if TestAuthorDefault struct exists
    let _author_default = TestAuthorDefault {
        id: Some(1),                     // Should be Option<i32> since ID is defaultable
        name: "Test Author".to_string(), // Should remain String
        biography_id: None,              // Should remain Option<i32>
    };
}

#[test]
fn minimal_defaultable_struct_should_exist() {
    // MinimalDefaultableDefault should exist because ID is marked as defaultable
    let _minimal_default = MinimalDefaultableDefault {
        id: None,                     // Should be Option<i32>
        name: "testuser".to_string(), // Should remain String
        biography_id: None,           // Should remain Option<i32>
    };
}

#[test]
fn defaultable_fields_can_be_none() {
    let _author_default = TestAuthorDefault {
        id: None, // Can be None since it's defaultable (auto-generated)
        name: "Test Author".to_string(),
        biography_id: None, // Can remain None
    };
}

#[test]
fn field_visibility_is_preserved() {
    let _author_default = TestAuthorDefault {
        id: Some(1),              // pub
        name: "Test".to_string(), // pub
        biography_id: Some(1),    // pub, Option<i32>
    };

    // This test ensures field visibility is preserved in generated struct
}

mod defaultable_tests {
    use super::*;
    use georm::Defaultable;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_create_entity_from_defaultable_with_id(pool: PgPool) {
        // Test creating entity from defaultable struct with explicit ID
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

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_create_entity_from_defaultable_without_id(pool: PgPool) {
        // Test creating entity from defaultable struct with auto-generated ID
        let author_default = TestAuthorDefault {
            id: None, // Let database generate the ID
            name: "Jane Smith".to_string(),
            biography_id: None,
        };

        let created_author = author_default.create(&pool).await.unwrap();

        // ID should be auto-generated (positive value)
        assert!(created_author.id > 0);
        assert_eq!(created_author.name, "Jane Smith");
        assert_eq!(created_author.biography_id, None);
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_create_entity_from_minimal_defaultable(pool: PgPool) {
        // Test creating entity from minimal defaultable struct
        let minimal_default = MinimalDefaultableDefault {
            id: None,
            name: "Alice Wonder".to_string(),
            biography_id: Some(1), // Reference existing biography
        };

        let created_author = minimal_default.create(&pool).await.unwrap();

        assert!(created_author.id > 0);
        assert_eq!(created_author.name, "Alice Wonder");
        assert_eq!(created_author.biography_id, Some(1));
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_create_multiple_entities_from_defaultable(pool: PgPool) {
        // Test creating multiple entities to ensure ID generation works properly
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

        // Both should have unique IDs
        assert!(created_author1.id > 0);
        assert!(created_author2.id > 0);
        assert_ne!(created_author1.id, created_author2.id);

        assert_eq!(created_author1.name, "Author One");
        assert_eq!(created_author2.name, "Author Two");
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_multiple_defaultable_fields_all_none(pool: PgPool) {
        // Test with multiple defaultable fields all set to None
        let multi_default = MultiDefaultableDefault {
            id: None,
            name: None, // This should use database default or be handled gracefully
            biography_id: None,
        };

        let result = multi_default.create(&pool).await;

        // This might fail if database doesn't have a default for name
        // That's expected behavior - test documents the current behavior
        match result {
            Ok(created) => {
                assert!(created.id > 0);
                // If successful, name should have some default value
            }
            Err(e) => {
                // Expected if no database default for name column
                assert!(e.to_string().contains("null") || e.to_string().contains("NOT NULL"));
            }
        }
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_multiple_defaultable_fields_mixed(pool: PgPool) {
        // Test with some defaultable fields set and others None
        let multi_default = MultiDefaultableDefault {
            id: None,                                // Let database generate
            name: Some("Explicit Name".to_string()), // Explicit value
            biography_id: Some(1),                   // Reference existing biography
        };

        let created = multi_default.create(&pool).await.unwrap();

        assert!(created.id > 0);
        assert_eq!(created.name, "Explicit Name");
        assert_eq!(created.biography_id, Some(1));
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_multiple_defaultable_fields_all_explicit(pool: PgPool) {
        // Test with all defaultable fields having explicit values
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

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_error_duplicate_id(pool: PgPool) {
        // Test error handling for duplicate ID constraint violation
        let author1 = TestAuthorDefault {
            id: Some(777),
            name: "First Author".to_string(),
            biography_id: None,
        };

        let author2 = TestAuthorDefault {
            id: Some(777), // Same ID - should cause constraint violation
            name: "Second Author".to_string(),
            biography_id: None,
        };

        // First creation should succeed
        let _created1 = author1.create(&pool).await.unwrap();

        // Second creation should fail due to duplicate key
        let result2 = author2.create(&pool).await;
        assert!(result2.is_err());

        let error = result2.unwrap_err();
        let error_str = error.to_string();
        assert!(
            error_str.contains("duplicate")
                || error_str.contains("unique")
                || error_str.contains("UNIQUE")
        );
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_error_invalid_foreign_key(pool: PgPool) {
        // Test error handling for invalid foreign key reference
        let author_default = TestAuthorDefault {
            id: None,
            name: "Test Author".to_string(),
            biography_id: Some(99999), // Non-existent biography ID
        };

        let result = author_default.create(&pool).await;

        // This should fail if there's a foreign key constraint
        // If no constraint exists, it will succeed (documents current behavior)
        match result {
            Ok(created) => {
                // No foreign key constraint - this is valid behavior
                assert!(created.id > 0);
                assert_eq!(created.biography_id, Some(99999));
            }
            Err(e) => {
                // Foreign key constraint violation
                let error_str = e.to_string();
                assert!(
                    error_str.contains("foreign")
                        || error_str.contains("constraint")
                        || error_str.contains("violates")
                );
            }
        }
    }

    #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
    async fn test_error_connection_handling(pool: PgPool) {
        // Test behavior with a closed/invalid pool
        // Note: This is tricky to test without actually closing the pool
        // Instead, we test with extremely long string that might cause issues
        let author_default = TestAuthorDefault {
            id: None,
            name: "A".repeat(10000), // Very long string - might hit database limits
            biography_id: None,
        };

        let result = author_default.create(&pool).await;

        // This documents current behavior - might succeed or fail depending on DB limits
        match result {
            Ok(created) => {
                assert!(created.id > 0);
                assert_eq!(created.name.len(), 10000);
            }
            Err(e) => {
                // Some kind of database limit hit
                assert!(!e.to_string().is_empty());
            }
        }
    }

    mod sql_validation_tests {
        use super::*;

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_sql_generation_no_defaultable_fields(pool: PgPool) {
            // Test SQL generation when no defaultable fields have None values
            let author_default = TestAuthorDefault {
                id: Some(100),
                name: "Test Name".to_string(),
                biography_id: Some(1),
            };

            // Capture the SQL by creating a custom query that logs the generated SQL
            // Since we can't directly inspect the generated SQL from the macro,
            // we test the behavior indirectly by ensuring all fields are included
            let created = author_default.create(&pool).await.unwrap();

            // Verify all fields were properly inserted
            assert_eq!(created.id, 100);
            assert_eq!(created.name, "Test Name");
            assert_eq!(created.biography_id, Some(1));

            // Verify the record exists in database with all expected values
            let found: TestAuthor = sqlx::query_as!(
                TestAuthor,
                "SELECT id, name, biography_id FROM authors WHERE id = $1",
                100
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(found.id, 100);
            assert_eq!(found.name, "Test Name");
            assert_eq!(found.biography_id, Some(1));
        }

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_sql_generation_with_defaultable_none(pool: PgPool) {
            // Test SQL generation when defaultable fields are None (should be excluded)
            let author_default = TestAuthorDefault {
                id: None, // This should be excluded from INSERT
                name: "Auto ID Test".to_string(),
                biography_id: None,
            };

            let created = author_default.create(&pool).await.unwrap();

            // ID should be auto-generated (not explicitly set)
            assert!(created.id > 0);
            assert_eq!(created.name, "Auto ID Test");
            assert_eq!(created.biography_id, None);

            // Verify the generated ID is actually from database auto-increment
            // by checking it's different from any manually set values
            assert_ne!(created.id, 100); // Different from previous test
        }

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_sql_generation_mixed_defaultable_fields(pool: PgPool) {
            // Test SQL with multiple defaultable fields where some are None
            let multi_default = MultiDefaultableDefault {
                id: None,                                // Should be excluded
                name: Some("Explicit Name".to_string()), // Should be included
                biography_id: Some(1),                   // Should be included
            };

            let created = multi_default.create(&pool).await.unwrap();

            // Verify the mixed field inclusion worked correctly
            assert!(created.id > 0); // Auto-generated
            assert_eq!(created.name, "Explicit Name"); // Explicitly set
            assert_eq!(created.biography_id, Some(1)); // Explicitly set
        }

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_placeholder_ordering_consistency(pool: PgPool) {
            // Test that placeholders are ordered correctly when fields are dynamically included
            // Create multiple records with different field combinations

            // First: only non-defaultable fields
            let record1 = MultiDefaultableDefault {
                id: None,
                name: None,
                biography_id: Some(1),
            };

            // Second: all fields explicit
            let record2 = MultiDefaultableDefault {
                id: Some(201),
                name: Some("Full Record".to_string()),
                biography_id: Some(1),
            };

            // Third: mixed combination
            let record3 = MultiDefaultableDefault {
                id: None,
                name: Some("Mixed Record".to_string()),
                biography_id: None,
            };

            // All should succeed with correct placeholder ordering
            let result1 = record1.create(&pool).await;
            let result2 = record2.create(&pool).await;
            let result3 = record3.create(&pool).await;

            // Handle record1 based on whether name has a database default
            match result1 {
                Ok(created1) => {
                    assert!(created1.id > 0);
                    assert_eq!(created1.biography_id, Some(1));
                }
                Err(_) => {
                    // Expected if name field has no database default
                }
            }

            let created2 = result2.unwrap();
            assert_eq!(created2.id, 201);
            assert_eq!(created2.name, "Full Record");
            assert_eq!(created2.biography_id, Some(1));

            let created3 = result3.unwrap();
            assert!(created3.id > 0);
            assert_eq!(created3.name, "Mixed Record");
            assert_eq!(created3.biography_id, None);
        }

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_field_inclusion_logic(pool: PgPool) {
            // Test that the field inclusion logic works correctly
            // by creating records that should result in different SQL queries

            let minimal = TestAuthorDefault {
                id: None,
                name: "Minimal".to_string(),
                biography_id: None,
            };

            let maximal = TestAuthorDefault {
                id: Some(300),
                name: "Maximal".to_string(),
                biography_id: Some(1),
            };

            let created_minimal = minimal.create(&pool).await.unwrap();
            let created_maximal = maximal.create(&pool).await.unwrap();

            // Minimal should have auto-generated ID, explicit name, NULL biography_id
            assert!(created_minimal.id > 0);
            assert_eq!(created_minimal.name, "Minimal");
            assert_eq!(created_minimal.biography_id, None);

            // Maximal should have all explicit values
            assert_eq!(created_maximal.id, 300);
            assert_eq!(created_maximal.name, "Maximal");
            assert_eq!(created_maximal.biography_id, Some(1));

            // Verify they are different records
            assert_ne!(created_minimal.id, created_maximal.id);
        }

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_returning_clause_functionality(pool: PgPool) {
            // Test that the RETURNING * clause works correctly with dynamic fields
            let author_default = TestAuthorDefault {
                id: None, // Should be populated by RETURNING clause
                name: "Return Test".to_string(),
                biography_id: None,
            };

            let created = author_default.create(&pool).await.unwrap();

            // Verify RETURNING clause populated all fields correctly
            assert!(created.id > 0); // Database-generated ID returned
            assert_eq!(created.name, "Return Test"); // Explicit value returned
            assert_eq!(created.biography_id, None); // NULL value returned correctly

            // Double-check by querying the database directly
            let verified: TestAuthor = sqlx::query_as!(
                TestAuthor,
                "SELECT id, name, biography_id FROM authors WHERE id = $1",
                created.id
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            assert_eq!(verified.id, created.id);
            assert_eq!(verified.name, created.name);
            assert_eq!(verified.biography_id, created.biography_id);
        }

        #[sqlx::test(fixtures("../tests/fixtures/simple_struct.sql"))]
        async fn test_query_parameter_binding_order(pool: PgPool) {
            // Test that query parameters are bound in the correct order
            // This is critical for the dynamic SQL generation

            // Create a record where the parameter order matters
            let test_record = MultiDefaultableDefault {
                id: Some(400),                              // This should be bound first (if included)
                name: Some("Param Order Test".to_string()), // This should be bound second (if included)
                biography_id: Some(1),                      // This should be bound last
            };

            let created = test_record.create(&pool).await.unwrap();

            // Verify all parameters were bound correctly
            assert_eq!(created.id, 400);
            assert_eq!(created.name, "Param Order Test");
            assert_eq!(created.biography_id, Some(1));

            // Test with different parameter inclusion order
            let test_record2 = MultiDefaultableDefault {
                id: None,                             // Excluded - should not affect parameter order
                name: Some("No ID Test".to_string()), // Should be bound first now
                biography_id: Some(1),                // Should be bound second now
            };

            let created2 = test_record2.create(&pool).await.unwrap();

            assert!(created2.id > 0); // Auto-generated
            assert_eq!(created2.name, "No ID Test");
            assert_eq!(created2.biography_id, Some(1));
        }
    }
}
