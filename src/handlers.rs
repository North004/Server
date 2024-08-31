use crate::{
    model::{Profile, Register, User},
    response::{ApiError, GeneralResponse, PostResponse, Status},
    schema::{CreatePostSchema, LikePostSchema, LoginUserSchema, RegisterUserSchema},
    AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde_json::json;
use std::sync::Arc;
use tower_sessions::Session;
use uuid::Uuid;
use validator::Validate;

pub async fn login_user_handler(
    session: Session,
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let user: Register = sqlx::query_as!(
        Register,
        "SELECT (id), (password) FROM users WHERE username = ($1)",
        body.username
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?
    .ok_or_else(|| ApiError::Fail("user does not exist".to_string()))?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(ApiError::Fail("incorrect password".to_string()));
    }

    session
        .insert("user_id", user.id)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    //WIERD FUNCTION
    let response = GeneralResponse::new(Status::Success, "user registerd", None);
    Ok(Json(response))
}

pub async fn logout_handler(session: Session) -> Result<impl IntoResponse, ApiError> {
    session
        .delete()
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "User logged out".to_string(),
        data: None,
    };
    Ok(Json(response))
}

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let user_exists: Option<bool> = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)",
        body.username.to_owned()
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    if let Some(exists) = user_exists {
        if exists {
            return Err(ApiError::Fail("username is taken".to_string()));
        }
    }

    let email_exists: Option<bool> = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        body.email
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    if let Some(exists) = email_exists {
        if exists {
            return Err(ApiError::Fail("Email is taken".to_string()));
        }
    }

    if body.validate().is_err() {
        return Err(ApiError::Fail("Email is not valid".to_string()));
    }
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|_| ApiError::InternalServerError)
        .map(|hash| hash.to_string())?;

    let tx = data
        .db
        .begin()
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let user_id: Uuid = sqlx::query_scalar!(
        "INSERT INTO users (username,email,password) VALUES ($1, $2, $3) RETURNING id",
        body.username.to_string(),
        body.email.to_string().to_ascii_lowercase(),
        hashed_password
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    sqlx::query!("INSERT INTO profiles (user_id, photo, bio) VALUES ($1, $2, $3) RETURNING id, user_id,photo,bio,created_at,updated_at",
        user_id,
        "default.jpg".to_string(),
        "".to_string(),
    )
    .fetch_one(&data.db)
    .await.map_err(|_| {
        ApiError::InternalServerError
    })?;

    tx.commit()
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "User registerd".to_string(),
        data: None,
    };
    Ok(Json(response))
}

pub async fn create_post(
    Extension(user): Extension<User>,
    State(data): State<Arc<AppState>>,
    Json(post): Json<CreatePostSchema>,
) -> Result<impl IntoResponse, ApiError> {
    sqlx::query!(
        "INSERT INTO posts (author_id,title,content) VALUES ($1,$2,$3)",
        user.id,
        post.title,
        post.content
    )
    .execute(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "Post created".to_string(),
        data: None,
    };

    Ok(Json(response))
}

pub async fn get_all_posts(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let posts: Vec<PostResponse> = sqlx::query_as!(
        PostResponse,
        "SELECT
            posts.id AS post_id,
            users.username AS author,
            posts.title,
            posts.content,
            posts.created_at,
            posts.updated_at,
            posts.author_id,
            profiles.photo AS author_pfp,
            COALESCE(SUM(CASE WHEN post_reactions.is_like = TRUE THEN 1 ELSE 0 END), 0) AS like_count,
            COALESCE(SUM(CASE WHEN post_reactions.is_like = FALSE THEN 1 ELSE 0 END), 0) AS dislike_count
        FROM posts
        JOIN users ON posts.author_id = users.id
        JOIN profiles ON profiles.user_id = users.id
        LEFT JOIN post_reactions ON posts.id = post_reactions.post_id
        GROUP BY posts.id, users.username, posts.title, posts.content, posts.created_at, posts.updated_at, users.id, profiles.photo
        ORDER BY posts.created_at DESC"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "All posts retrived".to_string(),
        data: Some(json!(posts)),
    };
    Ok(Json(response))
}

pub async fn get_profile(
    Path(username): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    // Query to find the user by username
    let user_id: Option<uuid::Uuid> =
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(&data.db)
            .await
            .map_err(|_| ApiError::InternalServerError)?;

    // If user is not found, return 404 Not Found
    let user_id = match user_id {
        Some(id) => id,
        _ => {
            return Err(ApiError::Fail("User not found".to_string()));
        }
    };

    // Query to find the profile by user_id
    let profile: Option<Profile> = sqlx::query_as!(
        Profile,
        "SELECT id, user_id, photo, bio, created_at, updated_at FROM profiles WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    // If profile is not found, return 404 Not Found
    let profile = match profile {
        Some(profile) => profile,
        None => {
            return Err(ApiError::Fail("Profile not found".to_string()));
        }
    };

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "Profile retrived".to_string(),
        data: Some(json!(profile)),
    };
    Ok(Json(response))
}

pub async fn get_all_users(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let users: Vec<User> = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "All users retrived".to_string(),
        data: Some(json!(users)),
    };

    Ok(Json(response))
}

pub async fn react_to_post(
    State(data): State<Arc<AppState>>,
    Extension(user): Extension<User>,
    Path(post_id): Path<Uuid>,
    Json(is_like): Json<LikePostSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let existing_reaction = sqlx::query!(
        "SELECT id FROM post_reactions WHERE post_id = $1 AND user_id = $2",
        post_id,
        user.id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    if let Some(reaction) = existing_reaction {
        // Update the existing reaction
        sqlx::query!(
            "UPDATE post_reactions SET is_like = $1 WHERE id = $2",
            is_like.is_like,
            reaction.id
        )
        .execute(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    } else {
        // Insert a new reaction
        sqlx::query!(
            "INSERT INTO post_reactions (post_id, user_id, is_like) VALUES ($1, $2, $3)",
            post_id,
            user.id,
            is_like.is_like
        )
        .execute(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    }

    let counts = sqlx::query!(
        "SELECT 
            COALESCE(SUM(CASE WHEN is_like = TRUE THEN 1 ELSE 0 END), 0) AS like_count,
            COALESCE(SUM(CASE WHEN is_like = FALSE THEN 1 ELSE 0 END), 0) AS dislike_count
        FROM post_reactions
        WHERE post_id = $1",
        post_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "Reaction Recorded".to_string(),
        data: Some(json!({
                "post_id": post_id,
                "like_count" : counts.like_count,
                "dislike_count" : counts.dislike_count,
        })),
    };
    Ok(Json(response))
}

pub async fn delete_post(
    Extension(user): Extension<User>,
    State(data): State<Arc<AppState>>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let post_uuid = sqlx::query_scalar!("SELECT author_id FROM posts WHERE id = $1", post_id)
        .fetch_one(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;
    if user.id != post_uuid {
        return Err(ApiError::Fail("not authorized to delete post".to_string()));
    }
    sqlx::query!("DELETE FROM posts WHERE id = $1", post_id)
        .execute(&data.db)
        .await
        .map_err(|_| ApiError::InternalServerError)?;

    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "Post delleted".to_string(),
        data: None,
    };

    Ok(Json(response))
}

pub async fn is_logged_in() -> Result<impl IntoResponse, ApiError> {
    let response: GeneralResponse = GeneralResponse {
        status: Status::Success,
        message: "User logged in".to_string(),
        data: Some(json!({
            "is_logged_in": true,
        })),
    };
    Ok(Json(response))
}
