use axum::body::Body;
use axum::response::Response;
use axum::{http::StatusCode, response::IntoResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};
use uuid::Uuid;
use axum::extract::{rejection::JsonRejection, FromRequest, MatchedPath, Request, State};


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
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "error")]
    Error,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralResponse {
    pub status: Status,
    pub message: String,
    #[serde(serialize_with = "serialize_option_value")]
    pub data: Option<Value>,
}

fn serialize_option_value<S>(option: &Option<Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match option {
        Some(value) => serializer.serialize_some(value),
        None => serializer.serialize_some(&Value::Array(vec![])),
    }
}

impl GeneralResponse {
    pub fn new(status: Status, message: &str, data: Option<Value>) -> Self {
        Self {
            status,
            message: message.to_string(),
            data,
        }
    }
}

pub enum ApiError {
    Fail(String),
    InternalServerError,
    JsonRejection(JsonRejection),
}


#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}


impl IntoResponse for ApiError {
    fn into_response(self) -> Response<Body> {
        let (status, message, statusj) = match self {
            ApiError::Fail(err) => (StatusCode::OK, err, Status::Fail),
            ApiError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                Status::Error,
            ),
            ApiError::JsonRejection(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR,
                "invalid json".to_string(),
                Status::Error)
            }
        };

        let json_response = GeneralResponse {
            status: statusj,
            message,
            data: None,
        };

        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(json!(json_response).to_string()))
            .unwrap()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}