use std::sync::Arc;

use axum::{ middleware, routing::{ get, post, delete }, Router };

use crate::{
    handlers::{
        create_post,
        get_all_posts,
        get_all_users,
        get_profile,
        login_user_handler,
        logout_handler,
        react_to_post,
        register_user_handler,
        delete_post,
    },
    jwt_auth::auth,
    AppState,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        .route("/api/users/:username", get(get_profile))
        .route(
            "/api/auth/logout",
            get(logout_handler).route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route("/api/users/get_all", get(get_all_users))
        .route(
            "/api/posts",
            post(create_post).route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route(
            "/api/posts/:post_id",
            delete(delete_post).route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route(
            "/api/posts/get_all",
            get(get_all_posts).route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .route(
            "/api/posts/:post_id/react",
            post(react_to_post).route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .with_state(app_state)
}
