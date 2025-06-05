use clap::{Parser, Subcommand};

mod comments;
mod followers;
mod users;

type Result = crate::Result<()>;

pub trait Executable {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result;
}

#[derive(Debug, Clone, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Executable for Cli {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        self.command.execute(pool).await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Users(users::UserArgs),
    Followers(followers::FollowersArgs),
    Comments(comments::CommentArgs),
}

impl Executable for Commands {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        match self {
            Commands::Users(user_args) => user_args.execute(pool).await,
            Commands::Followers(followers_args) => followers_args.execute(pool).await,
            Commands::Comments(comment_args) => comment_args.execute(pool).await,
        }
    }
}
