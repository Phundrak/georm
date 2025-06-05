use super::{Executable, Result};
use crate::{errors::UserInputError, models::User};
use clap::{Args, Subcommand};
use georm::Georm;
use inquire::{max_length, min_length, required};

#[derive(Debug, Args, Clone)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

impl Executable for UserArgs {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        self.command.execute(pool).await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum UserCommand {
    Add { username: Option<String> },
    Remove { id: Option<i32> },
    UpdateProfile { id: Option<i32> },
    List,
}

impl Executable for UserCommand {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        match self {
            UserCommand::Add { username } => add_user(username.clone(), pool).await,
            UserCommand::Remove { id } => remove_user(*id, pool).await,
            UserCommand::UpdateProfile { id } => update_profile(*id, pool).await,
            UserCommand::List => list_all(pool).await,
        }
    }
}

async fn add_user(username: Option<String>, pool: &sqlx::PgPool) -> Result {
    let username = match username {
        Some(username) => username,
        None => inquire::Text::new("Enter a username:")
            .prompt()
            .map_err(|_| UserInputError::InputRequired)?,
    };
    let user = User::try_new(&username, pool).await?;
    println!("The user {user} has been created!");
    Ok(())
}

async fn remove_user(id: Option<i32>, pool: &sqlx::PgPool) -> Result {
    let user = User::remove_interactive(id, pool).await?;
    println!("Removed user {user} from database");
    Ok(())
}

async fn update_profile(id: Option<i32>, pool: &sqlx::PgPool) -> Result {
    let (user, mut profile) = User::update_profile(id, pool).await?;
    let update_display_name = inquire::Confirm::new(
        format!(
            "Your current display name is \"{}\", do you want to update it?",
            profile.get_display_name()
        )
        .as_str(),
    )
    .with_default(false)
    .prompt()
    .map_err(UserInputError::InquireError)?;
    let display_name = if update_display_name {
        Some(
            inquire::Text::new("New display name:")
                .with_help_message("Your display name should not exceed 100 characters")
                .with_validator(min_length!(3))
                .with_validator(max_length!(100))
                .with_validator(required!())
                .prompt()
                .map_err(UserInputError::InquireError)?,
        )
    } else {
        Some(profile.get_display_name())
    };
    let update_bio = inquire::Confirm::new(
        format!(
            "Your current bio is:\n===\n{}\n===\nDo you want to update it?",
            profile.get_bio()
        )
        .as_str(),
    )
    .with_default(false)
    .prompt()
    .map_err(UserInputError::InquireError)?;
    let bio = if update_bio {
        Some(
            inquire::Text::new("New bio:")
                .with_validator(min_length!(0))
                .prompt()
                .map_err(UserInputError::InquireError)?,
        )
    } else {
        Some(profile.get_bio())
    };
    let profile = profile.update_interactive(display_name, bio, pool).await?;
    println!("Profile of {user} updated:\n{profile}");
    Ok(())
}

async fn list_all(pool: &sqlx::PgPool) -> Result {
    let users = User::find_all(pool).await?;
    println!("List of users:\n");
    for user in users {
        println!("{user}");
    }
    Ok(())
}
