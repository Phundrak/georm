use super::{Executable, Result};
use crate::models::{FollowerDefault, User};
use clap::{Args, Subcommand};
use georm::Defaultable;
use std::collections::HashMap;

#[derive(Debug, Args, Clone)]
pub struct FollowersArgs {
    #[command(subcommand)]
    pub command: FollowersCommand,
}

impl Executable for FollowersArgs {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        self.command.execute(pool).await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum FollowersCommand {
    Follow {
        follower: Option<String>,
        followed: Option<String>,
    },
    Unfollow {
        follower: Option<String>,
    },
    ListFollowers {
        user: Option<String>,
    },
    ListFollowed {
        user: Option<String>,
    },
}

impl Executable for FollowersCommand {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        match self {
            FollowersCommand::Follow { follower, followed } => {
                follow_user(follower.clone(), followed.clone(), pool).await
            }
            FollowersCommand::Unfollow { follower } => unfollow_user(follower.clone(), pool).await,
            FollowersCommand::ListFollowers { user } => {
                list_user_followers(user.clone(), pool).await
            }
            FollowersCommand::ListFollowed { user } => list_user_followed(user.clone(), pool).await,
        }
    }
}

async fn follow_user(
    follower: Option<String>,
    followed: Option<String>,
    pool: &sqlx::PgPool,
) -> Result {
    let follower = User::get_user_by_username_or_select(
        follower.as_deref(),
        "Select who will be following someone:",
        pool,
    )
    .await?;
    let followed = User::get_user_by_username_or_select(
        followed.as_deref(),
        "Select who will be followed:",
        pool,
    )
    .await?;
    let follow = FollowerDefault {
        id: None,
        follower: follower.id,
        followed: followed.id,
    };
    follow.create(pool).await?;
    println!("User {follower} now follows {followed}");
    Ok(())
}

async fn unfollow_user(follower: Option<String>, pool: &sqlx::PgPool) -> Result {
    let follower =
        User::get_user_by_username_or_select(follower.as_deref(), "Select who is following", pool)
            .await?;
    let followed_list: HashMap<String, User> = follower
        .get_followed(pool)
        .await?
        .iter()
        .map(|person| (person.username.clone(), person.clone()))
        .collect();
    let followed = inquire::Select::new(
        "Who to unfollow?",
        followed_list.clone().into_keys().collect(),
    )
    .prompt()
    .unwrap();
    let followed = followed_list.get(&followed).unwrap();
    sqlx::query!(
        "DELETE FROM Followers WHERE follower = $1 AND followed = $2",
        follower.id,
        followed.id
    )
    .execute(pool)
    .await?;
    println!("User {follower} unfollowed {followed}");
    Ok(())
}

async fn list_user_followers(user: Option<String>, pool: &sqlx::PgPool) -> Result {
    let user = User::get_user_by_username_or_select(
        user.as_deref(),
        "Whose followers do you want to display?",
        pool,
    )
    .await?;
    println!("List of followers of {user}:\n");
    user.get_followers(pool)
        .await?
        .iter()
        .for_each(|person| println!("{person}"));
    Ok(())
}

async fn list_user_followed(user: Option<String>, pool: &sqlx::PgPool) -> Result {
    let user = User::get_user_by_username_or_select(
        user.as_deref(),
        "Whose follows do you want to display?",
        pool,
    )
    .await?;
    println!("List of people followed by {user}:\n");
    user.get_followed(pool)
        .await?
        .iter()
        .for_each(|person| println!("{person}"));
    Ok(())
}
