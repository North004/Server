use crate::model::User;
use crate::response::ApiError;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;

pub async fn auth(
    session: Session,
    State(data): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(user_id) = session
        .get::<Uuid>("user_id")
        .await
        .map_err(|_| ApiError::InternalServerError)?
    {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
            .fetch_optional(&data.db)
            .await
            .map_err(|_| ApiError::InternalServerError)?;

        let user = user.ok_or_else(|| ApiError::Fail("user unauthorized".to_string()))?;

        req.extensions_mut().insert(user);
        Ok(next.run(req).await)
    } else {
        Err(ApiError::Fail("user unathorized".to_string()))
    }
}
