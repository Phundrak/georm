use std::collections::HashMap;

use crate::{Result, errors::UserInputError};
use georm::{Defaultable, Georm};

use super::{Comment, Profile};

#[derive(Debug, Georm, Clone)]
#[georm(
    table = "Users",
    one_to_one = [{
        name = "profile", remote_id = "user_id", table = "Profiles", entity = Profile
    }],
    one_to_many = [{
        name = "comments", remote_id = "author_id", table = "Comments", entity = Comment
    }],
    many_to_many = [{
        name = "followers",
        table = "Users",
        entity = User,
        link = { table = "Followers", from = "followed", to = "follower" }
    },
{
        name = "followed",
        table = "Users",
        entity = User,
        link = { table = "Followers", from = "follower", to = "followed" }
    }
    ]
)]
pub struct User {
    #[georm(id, defaultable)]
    pub id: i32,
    pub username: String,
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.username, self.id)
    }
}

impl From<&str> for UserDefault {
    fn from(value: &str) -> Self {
        Self {
            id: None,
            username: value.to_string(),
        }
    }
}

impl User {
    async fn select_user<'e, E>(prompt: &str, executor: E) -> Result<Self>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let users: HashMap<String, Self> = Self::find_all(executor)
            .await?
            .into_iter()
            .map(|user| (user.username.clone(), user))
            .collect();
        let username = inquire::Select::new(prompt, users.clone().into_keys().collect())
            .prompt()
            .map_err(UserInputError::InquireError)?;
        let user: &Self = users.get(&username).unwrap();
        Ok(user.clone())
    }

    pub async fn get_user_by_id_or_select<'e, E>(
        id: Option<i32>,
        prompt: &str,
        executor: E
    ) -> Result<Self>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let user = match id {
            Some(id) => Self::find(executor, &id)
                .await?
                .ok_or(UserInputError::UserDoesNotExist)?,
            None => Self::select_user(prompt, executor).await?,
        };
        Ok(user)
    }

    pub async fn get_user_by_username_or_select<'e, E>(
        username: Option<&str>,
        prompt: &str,
        executor: E,
    ) -> Result<Self>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let user = match username {
            Some(username) => Self::find_by_username(username, executor)
                .await?
                .ok_or(UserInputError::UserDoesNotExist)?,
            None => Self::select_user(prompt, executor).await?,
        };
        Ok(user)
    }

    pub async fn find_by_username<'e, E>(username: &str, executor: E) -> Result<Option<Self>>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        sqlx::query_as!(
            Self,
            "SELECT * FROM Users u WHERE u.username = $1",
            username
        )
        .fetch_optional(executor)
        .await
        .map_err(UserInputError::DatabaseError)
    }

    pub async fn try_new(username: &str, pool: &sqlx::PgPool) -> Result<Self> {
        let user = UserDefault::from(username);
        user.create(pool)
            .await
            .map_err(UserInputError::DatabaseError)
    }

    pub async fn remove_interactive(id: Option<i32>, pool: &sqlx::PgPool) -> Result<Self> {
        let prompt = "Select a user to delete:";
        let user = Self::get_user_by_id_or_select(id, prompt, pool).await?;
        let _ = user.clone().delete(pool).await?;
        Ok(user)
    }

    pub async fn update_profile(id: Option<i32>, pool: &sqlx::PgPool) -> Result<(User, Profile)>
    {
        let prompt = "Select the user whose profile you want to update";
        let user = Self::get_user_by_id_or_select(id, prompt, pool).await?;
        let profile = match user.get_profile(pool).await? {
            Some(profile) => profile,
            None => Profile::try_new(user.id, pool).await?,
        };
        Ok((user, profile))
    }
}
