use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserSchema {
    pub username: String,
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostSchema {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct LikePostSchema {
    pub is_like: bool
}
