use super::{Executable, Result};
use crate::{
    errors::UserInputError,
    models::{Comment, CommentDefault, User},
};
use clap::{Args, Subcommand};
use georm::{Defaultable, Georm};
use std::collections::HashMap;

#[derive(Debug, Args, Clone)]
pub struct CommentArgs {
    #[command(subcommand)]
    pub command: CommentCommand,
}

impl Executable for CommentArgs {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        self.command.execute(pool).await
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum CommentCommand {
    Create {
        text: Option<String>,
        username: Option<String>,
    },
    Remove {
        id: Option<i32>,
    },
    RemoveFromUser {
        username: Option<String>,
    },
    ListFromUser {
        username: Option<String>,
    },
    List,
}

impl Executable for CommentCommand {
    async fn execute(&self, pool: &sqlx::PgPool) -> Result {
        match self {
            CommentCommand::Create { text, username } => {
                create_comment(username.clone(), text.clone(), pool).await
            }
            CommentCommand::Remove { id } => remove_comment(*id, pool).await,
            CommentCommand::RemoveFromUser { username } => {
                remove_user_comment(username.clone(), pool).await
            }
            CommentCommand::ListFromUser { username } => {
                list_user_comments(username.clone(), pool).await
            }
            CommentCommand::List => list_comments(pool).await,
        }
    }
}

async fn create_comment(
    username: Option<String>,
    text: Option<String>,
    pool: &sqlx::PgPool,
) -> Result {
    let prompt = "Who is creating the comment?";
    let user = User::get_user_by_username_or_select(username.as_deref(), prompt, pool).await?;
    let content = match text {
        Some(text) => text,
        None => inquire::Text::new("Content of the comment:")
            .prompt()
            .map_err(UserInputError::InquireError)?,
    };
    let comment = CommentDefault {
        author_id: user.id,
        content,
        id: None,
    };
    let comment = comment.create(pool).await?;
    println!("Successfuly created comment:\n{comment}");
    Ok(())
}

async fn remove_comment(id: Option<i32>, pool: &sqlx::PgPool) -> Result {
    let prompt = "Select the comment to remove:";
    let comment = match id {
        Some(id) => Comment::find(pool, &id)
            .await
            .map_err(UserInputError::DatabaseError)?
            .ok_or(UserInputError::CommentDoesNotExist)?,
        None => Comment::select_comment(prompt, pool).await?,
    };
    comment.delete(pool).await?;
    Ok(())
}

async fn remove_user_comment(username: Option<String>, pool: &sqlx::PgPool) -> Result {
    let prompt = "Select user whose comment you want to delete:";
    let user = User::get_user_by_username_or_select(username.as_deref(), prompt, pool).await?;
    let comments: HashMap<String, Comment> = user
        .get_comments(pool)
        .await?
        .into_iter()
        .map(|comment| (comment.content.clone(), comment))
        .collect();
    let selected_comment_content =
        inquire::Select::new(prompt, comments.clone().into_keys().collect())
            .prompt()
            .map_err(UserInputError::InquireError)?;
    let comment: &Comment = comments.get(&selected_comment_content).unwrap();
    comment.delete(pool).await?;
    Ok(())
}

async fn list_user_comments(username: Option<String>, pool: &sqlx::PgPool) -> Result {
    let prompt = "User whose comment you want to list:";
    let user = User::get_user_by_username_or_select(username.as_deref(), prompt, pool).await?;
    println!("List of comments from user:\n");
    for comment in user.get_comments(pool).await? {
        println!("{comment}\n");
    }
    Ok(())
}

async fn list_comments(pool: &sqlx::PgPool) -> Result {
    let comments = Comment::find_all(pool).await?;
    println!("List of all comments:\n");
    for comment in comments {
        println!("{comment}\n")
    }
    Ok(())
}
