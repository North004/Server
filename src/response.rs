use chrono::{DateTime, Utc};
use serde::Serialize;
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
