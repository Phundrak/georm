use georm::Georm;

mod models;
use models::{UserRole, UserRoleId};

#[sqlx::test(fixtures("composite_key"))]
async fn composite_key_find(pool: sqlx::PgPool) -> sqlx::Result<()> {
    // This will test the find query generation bug
    let id = models::UserRoleId {
        user_id: 1,
        role_id: 1,
    };

    let result = UserRole::find(&pool, &id).await?;
    assert!(result.is_some());

    let user_role = result.unwrap();
    assert_eq!(1, user_role.user_id);
    assert_eq!(1, user_role.role_id);

    Ok(())
}

#[test]
fn composite_key_get_id() {
    let user_role = UserRole {
        user_id: 1,
        role_id: 1,
        assigned_at: chrono::Local::now().into(),
    };

    // This will test the get_id implementation bug
    let id = user_role.get_id();
    assert_eq!(1, id.user_id);
    assert_eq!(1, id.role_id);
}

#[sqlx::test(fixtures("composite_key"))]
async fn composite_key_create_or_update(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let new_user_role = UserRole {
        user_id: 5,
        role_id: 2,
        assigned_at: chrono::Local::now().into(),
    };

    // This will test the upsert query generation bug
    let result = new_user_role.create_or_update(&pool).await?;
    assert_eq!(5, result.user_id);
    assert_eq!(2, result.role_id);

    Ok(())
}

#[sqlx::test(fixtures("composite_key"))]
async fn composite_key_delete(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let id = models::UserRoleId {
        user_id: 1,
        role_id: 1,
    };

    let rows_affected = UserRole::delete_by_id(&pool, &id).await?;
    assert_eq!(1, rows_affected);

    // Verify it's deleted
    let result = UserRole::find(&pool, &id).await?;
    assert!(result.is_none());

    Ok(())
}

#[sqlx::test(fixtures("composite_key"))]
async fn composite_key_find_all(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let all_user_roles = UserRole::find_all(&pool).await?;
    assert_eq!(4, all_user_roles.len());
    Ok(())
}

#[sqlx::test(fixtures("composite_key"))]
async fn composite_key_create(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let new_user_role = UserRole {
        user_id: 10,
        role_id: 5,
        assigned_at: chrono::Local::now().into(),
    };
    let result = new_user_role.create(&pool).await?;
    assert_eq!(new_user_role.user_id, result.user_id);
    assert_eq!(new_user_role.role_id, result.role_id);
    Ok(())
}

#[sqlx::test(fixtures("composite_key"))]
async fn composite_key_update(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let mut user_role = UserRole::find(
        &pool,
        &UserRoleId {
            user_id: 1,
            role_id: 1,
        },
    )
    .await?
    .unwrap();
    let now: chrono::DateTime<chrono::Utc> = chrono::Local::now().into();
    user_role.assigned_at = now;
    let updated = user_role.update(&pool).await?;
    assert_eq!(
        now.timestamp_millis(),
        updated.assigned_at.timestamp_millis()
    );
    assert_eq!(1, updated.user_id);
    assert_eq!(1, updated.role_id);
    Ok(())
}
