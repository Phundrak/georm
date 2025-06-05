mod cli;
mod errors;
mod models;

use clap::Parser;
use cli::{Cli, Executable};

type Result<T> = std::result::Result<T, errors::UserInputError>;

#[tokio::main]
async fn main() {
    let args = Cli::parse();
    let url = std::env::var("DATABASE_URL").expect("Environment variable DATABASE_URL must be set");
    let pool =
        sqlx::PgPool::connect_lazy(url.as_str()).expect("Failed to create database connection");
    match args.command.execute(&pool).await {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {e}"),
    }
}
