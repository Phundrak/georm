use super::User;
use georm::Georm;

#[derive(Debug, Clone, Georm)]
#[georm(table = "Followers")]
pub struct Follower {
    #[georm(id, defaultable)]
    pub id: i32,
    #[georm(relation = {
        entity = User,
        table = "Users",
        name = "followed"
    })]
    pub followed: i32,
    #[georm(relation = {
        entity = User,
        table = "Users",
        name = "follower"
    })]
    pub follower: i32,
}
