use std::sync::Arc;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use serde_json::json;
use validator::Validate;

use crate::{
    errors::ApiError,
    model::{Post, Profile, TokenClaims, User},
    response::{FilteredUser, PostResponse,ApiResponse,TokenResponse,UserResponse},
    schema::{CreatePostSchema, LoginUserSchema, RegisterUserSchema},
    AppState,
    config::Config
};

pub async fn logout_handler() -> Result<impl IntoResponse, ApiError> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);
    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let user = sqlx::query!(
        "SELECT * FROM users WHERE username = $1",
        body.username
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?
    .ok_or_else(|| ApiError::BadRequest("User does not exist".to_owned()))?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(ApiError::BadRequest("Invalid password".to_owned()));
    }
    let exptime: i64 = data.env.jwt_expires_in.parse().map_err(|e| ApiError::InternalServerError)?;

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(exptime)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::minutes(exptime))
        .same_site(SameSite::Lax)
        .http_only(true);

    let token_response =  TokenResponse { token };
    let json_response: ApiResponse<TokenResponse> = ApiResponse {
        status: "Success",
        message: "Jwt Token",
        data: Some(token_response)
    };
    let mut response = Response::new(json!(json_response).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(body.username.to_owned())
            .fetch_one(&data.db)
            .await
            .map_err(|e| ApiError::InternalServerError)?;

    if let Some(exists) = user_exists {
        if exists {
            return Err(ApiError::BadRequest("Username is taken".to_owned()));
        }
    }

    if let Err(validation_errors) = body.validate() {
        return Err(ApiError::BadRequest("Email is invalid".to_owned()));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalServerError)
        .map(|hash| hash.to_string())?;

    let user: User = sqlx::query_as!(User,
        "INSERT INTO users (username,email,password) VALUES ($1, $2, $3) RETURNING ALL",
        body.username.to_string(),
        body.email.to_string().to_ascii_lowercase(),
        hashed_password
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?;

    sqlx::query_as!(Profile,
        "INSERT INTO profiles (user_id, photo, bio) VALUES ($1, $2, $3) RETURNING id, user_id,photo,bio,created_at,updated_at",
        user.id,
        "default.png".to_string(),
        "My Bio".to_string(),
    )
    .fetch_one(&data.db)
    .await.map_err(|e| {
        ApiError::InternalServerError
    })?;

    let user_response =  ApiResponse {
        status: "Success",
        message: "User Created",
        data: Some(user.username)
    };
    

    Ok(Json(user_response))
}


//real function lets check it Checked
pub async fn create_post(
    Extension(user): Extension<User>,
    State(data): State<Arc<AppState>>,
    Json(post): Json<CreatePostSchema>,
) -> Result<impl IntoResponse, ApiError> {
    sqlx::query!(
        "INSERT INTO posts (user_id,title,content) VALUES ($1,$2,$3)",
        user.id,
        post.title,
        post.content
    )
    .execute(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?;

    let response: ApiResponse<()> = ApiResponse {
        status: "Success",
        message: "Post Created",
        data: None,
    };

    Ok(Json(response))
}

//gets all posts
pub async fn get_all_posts(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let post = sqlx::query_as!(
        PostResponse,
        "SELECT posts.title, posts.content, posts.created_at,posts.updated_at,users.username FROM posts JOIN users ON posts.user_id = users.id"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?;
    Ok(Json(post))
}

//function has been checked
pub async fn get_profile(
    Path(username): Path<String>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    // Query to find the user by username
    let user_id: Option<uuid::Uuid> =
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(&data.db)
            .await
            .map_err(|e| ApiError::InternalServerError)?;

    // If user is not found, return 404 Not Found
    let user_id = match user_id {
        Some(id) => id,
        None => {
            return Err(ApiError::NotFound("User Not Found".to_owned()));
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
    .map_err(|e| ApiError::InternalServerError)?;

    // If profile is not found, return 404 Not Found
    let profile = match profile {
        Some(profile) => profile,
        None => {
            return Err(ApiError::NotFound("Profile Not Found".to_owned()));
        }
    };

    let response = ApiResponse {
        status: "Success",
        message: "Returns User Profile",
        data: Some(profile)
    };

    Ok(Json(response))
}


//test function
pub async fn get_all_users(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let users: Vec<User> = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&data.db)
        .await
        .map_err(|e| ApiError::InternalServerError)?;
    let response = ApiResponse {
        status: "Success",
        message: "Returns User Profile",
        data: Some(users)
    };
    Ok(Json(response))
}
