use crate::{
    config::Config,
    errors::ApiError,
    model::{AlterdPost, Post, Profile, TokenClaims, User},
    schema::{CreatePostSchema, LoginUserSchema, RegisterUserSchema,LikePostSchema},
    AppState,
    filters::filter_user,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{Path, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use serde_json::json;
use std::sync::Arc; 
use uuid::Uuid;
use validator::Validate;

pub async fn logout_handler() -> Result<impl IntoResponse, ApiError> {
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);
    let mut response = Response::new(
        json!({"status": "Success","message" : "User has been logged out"}).to_string(),
    );
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(response)
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, ApiError> {
    let user: User = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        body.username
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?
    .ok_or_else(|| ApiError::BadRequest("No user exists with this name".to_owned()))?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(ApiError::BadRequest("Incorrect password".to_owned()));
    }

    let exptime: i64 = data
        .env
        .jwt_expires_in
        .parse()
        .map_err(|e| ApiError::InternalServerError)?;

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

    let json_response = json!({
        "status" : "Success",
        "message" : "User has been logged in",
        "user" : filter_user(&user),
    });

    let mut response = Response::new(json_response.to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
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
    .map_err(|e| ApiError::InternalServerError)?;

    if let Some(exists) = user_exists {
        if exists {
            return Err(ApiError::BadRequest(
                "Username already exists".to_owned(),
            ));
        }
    }

    let email_exists: Option<bool> = sqlx::query_scalar!(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
        body.email
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?;

    if let Some(exists) = email_exists {
        if exists {
            return Err(ApiError::BadRequest("Email is already in use".to_owned()));
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

    let user_id: Uuid = sqlx::query_scalar!(
        "INSERT INTO users (username,email,password) VALUES ($1, $2, $3) RETURNING id",
        body.username.to_string(),
        body.email.to_string().to_ascii_lowercase(),
        hashed_password
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?;

    sqlx::query_as!(Profile,
        "INSERT INTO profiles (user_id, photo, bio) VALUES ($1, $2, $3) RETURNING id, user_id,photo,bio,created_at,updated_at",
        user_id,
        "default.png".to_string(),
        "My Bio".to_string(),
    )
    .fetch_one(&data.db)
    .await.map_err(|e| {
        ApiError::InternalServerError
    })?;

    let response = json!({
        "status" : "Success",
        "message" : "User has been registerd"
    });
    Ok(Json(response))
}

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

    let response = json!({
        "status" : "Success",
        "message" : "Post has been created"
    });

    Ok(Json(response))
}

pub async fn get_all_posts(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let posts: Vec<AlterdPost> = sqlx::query_as!(
         AlterdPost,
        "SELECT 
            posts.id, 
            users.username, 
            posts.title, 
            posts.content, 
            posts.created_at, 
            posts.updated_at,
            users.id AS user_id,
            COALESCE(SUM(CASE WHEN post_reactions.is_like = TRUE THEN 1 ELSE 0 END), 0) AS like_count,
            COALESCE(SUM(CASE WHEN post_reactions.is_like = FALSE THEN 1 ELSE 0 END), 0) AS dislike_count
        FROM posts
        JOIN users ON posts.user_id = users.id
        LEFT JOIN post_reactions ON posts.id = post_reactions.post_id
        GROUP BY posts.id, users.username, posts.title, posts.content, posts.created_at, posts.updated_at, users.id
        ORDER BY posts.created_at DESC"
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError)?;

    let response = json!({
        "status" : "Success",
        "message" : "All posts retrived",
        "posts" : posts
    });
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

    let response = json!({
        "status" : "Success",
        "message" : "Profile retrived",
        "profile": profile
    });
    Ok(Json(response))
}

pub async fn get_all_users(
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, ApiError> {
    let users: Vec<User> = sqlx::query_as!(User, "SELECT * FROM users")
        .fetch_all(&data.db)
        .await
        .map_err(|e| ApiError::InternalServerError)?;

    let response = json!({
        "status" : "Success",
        "message" : "all users retrived",
        "users" : users
    });

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
    .map_err(|e| ApiError::InternalServerError)?;

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
    }
    else {
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
    .map_err(|_| ApiError::NotFound("Error sorry".to_owned()))?;

    let response = json!({
        "status": "Success",
        "message": "Reaction recorded",
        "post" : json!({
                "postId": post_id,
                "likeCount" : counts.like_count,
                "dislikeCount" : counts.dislike_count,
        })
    });
    Ok(Json(response))
}


pub async fn is_loggedin(Extension(user): Extension<User>,) -> Result<impl IntoResponse,ApiError> {
    Ok(StatusCode::OK)
}
