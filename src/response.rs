use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct PostResponse {
    pub post_id: Uuid,
    pub author_id: Uuid,
    pub author: String,
    pub author_pfp: String,
    pub title: String,
    pub content: String,
    pub like_count: Option<i64>,
    pub dislike_count: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub enum Status {
    Success,
    Error,
}
#[derive(Serialize, Deserialize)]
pub struct GeneralResponse<T: Serialize> {
    pub status: Status,
    pub message: String,
    pub data: Option<T>,
}
