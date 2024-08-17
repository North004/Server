use chrono::prelude::*;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Profile {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub photo: Option<String>,
    pub bio: Option<String>,       // bio is optional in the SQL schema
    pub created_at: DateTime<Utc>, // added to match SQL schema
    pub updated_at: DateTime<Utc>, // added to match SQL schema
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Post {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>, // added to match SQL schema
    pub updated_at: DateTime<Utc>, // added to match SQL schema
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

