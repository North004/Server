use chrono::prelude::*;
use serde::{Serialize,Deserialize};
use serde_json::Value;

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct PostResponse {
    pub username: String,
    pub title: String,
    pub content: String,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub status: &'static str,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>
}

#[derive(Serialize, Debug, Deserialize)]
pub struct TokenResponse {
    pub token: String
}

#[derive(Serialize, Debug, Deserialize)]
pub struct UserResponse {
    pub username: String,
    pub created_at: DateTime<Utc>,
}
