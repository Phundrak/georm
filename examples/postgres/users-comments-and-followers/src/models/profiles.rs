use super::User;
use crate::{Result, errors::UserInputError};
use georm::{Defaultable, Georm};

#[derive(Debug, Georm, Default)]
#[georm(table = "Profiles")]
pub struct Profile {
    #[georm(id, defaultable)]
    pub id: i32,
    #[georm(relation = {
        entity = User,
        table = "Users",
        name = "user",
        nullable = false
    })]
    pub user_id: i32,
    pub bio: Option<String>,
    pub display_name: Option<String>,
}

impl std::fmt::Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Display Name:\t{}\nBiography:\n{}\n",
            self.get_display_name(),
            self.get_bio()
        )
    }
}

impl Profile {
    pub fn get_display_name(&self) -> String {
        self.display_name.clone().unwrap_or_default()
    }

    pub fn get_bio(&self) -> String {
        self.bio.clone().unwrap_or_default()
    }

    pub async fn try_new(user_id: i32, pool: &sqlx::PgPool) -> Result<Self> {
        let profile = ProfileDefault {
            user_id,
            id: None,
            bio: None,
            display_name: None,
        };
        profile
            .create(pool)
            .await
            .map_err(UserInputError::DatabaseError)
    }

    pub async fn update_interactive(
        &mut self,
        display_name: Option<String>,
        bio: Option<String>,
        pool: &sqlx::PgPool,
    ) -> Result<Self> {
        self.display_name = display_name;
        self.bio = bio;
        self.update(pool)
            .await
            .map_err(UserInputError::DatabaseError)
    }
}
