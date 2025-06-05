use super::User;
use crate::{Result, errors::UserInputError};
use georm::Georm;
use std::collections::HashMap;

#[derive(Debug, Georm, Clone)]
#[georm(table = "Comments")]
pub struct Comment {
    #[georm(id, defaultable)]
    pub id: i32,
    #[georm(relation = {
        entity = User,
        table = "Users",
        name = "author"
    })]
    pub author_id: i32,
    pub content: String,
}

impl Comment {
    pub async fn select_comment(prompt: &str, pool: &sqlx::PgPool) -> Result<Self> {
        let comments: HashMap<String, Self> = Self::find_all(pool)
            .await?
            .into_iter()
            .map(|comment| (comment.content.clone(), comment))
            .collect();
        let comment_content = inquire::Select::new(prompt, comments.clone().into_keys().collect())
            .prompt()
            .map_err(UserInputError::InquireError)?;
        let comment: &Self = comments.get(&comment_content).unwrap();
        Ok(comment.clone())
    }
}

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Comment:\nID:\t{}\nAuthor:\t{}\nContent:\t{}",
            self.id, self.author_id, self.content
        )
    }
}
