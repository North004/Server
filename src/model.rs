use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Register {
    pub id: Uuid,
    pub password: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Profile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub photo: String,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Post {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

//#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
//pub struct AlterdPost {
//  pub id: uuid::Uuid,
// #[serde(rename = "authorId")]
//pub user_id: uuid::Uuid,
// #[serde(rename = "author")]
//pub username: String,
//pub title: String,
//pub content: String,
//#[serde(rename = "likeCount")]
//pub like_count: Option<i64>,
//#[serde(rename = "dislikeCount")]
//pub dislike_count: Option<i64>,
//#[serde(rename = "createdAt")]
//pub created_at: DateTime<Utc>, // added to match SQL schema
//#[serde(rename = "updatedAt")]
//pub updated_at: DateTime<Utc>, // added to match SQL schema
//}

//#[derive(Debug, Serialize, Deserialize)]
//pub struct TokenClaims {
//  pub sub: String,
// pub iat: usize,
//pub exp: usize,
//}
