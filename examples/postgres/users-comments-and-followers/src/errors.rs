use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserInputError {
    #[error("Input required")]
    InputRequired,
    #[error("User ID does not exist")]
    UserDoesNotExist,
    #[error("Comment does not exist")]
    CommentDoesNotExist,
    #[error("Unexpected error, please try again")]
    InquireError(#[from] inquire::error::InquireError),
    #[error("Error from database: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
