use crate::{
    handlers::{
        create_post, delete_post, get_all_posts, get_all_users, get_profile, is_logged_in,
        login_user_handler, logout_handler, react_to_post, register_user_handler,
    },
    session_auth::auth,
    AppState,
};
use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    // Define the protected routes
    let protected_routes = Router::new()
        .route("/post", post(create_post))
        .route("/post/:post_id", delete(delete_post))
        .route("/post/:post_id/react", post(react_to_post))
        .route("/auth/logout", get(logout_handler))
        .route("/auth/is_logged_in", get(is_logged_in));

    // Define the unprotected routes
    let unprotected_routes = Router::new()
        .route("/user/:username", get(get_profile))
        .route("/user/get_all", get(get_all_users))
        .route("/post/get_all", get(get_all_posts))
        .route("/auth/login", post(login_user_handler))
        .route("/auth/register", post(register_user_handler));

    // Apply the middleware layer to protected routes
    let protected_routes_with_auth =
        protected_routes.layer(middleware::from_fn_with_state(app_state.clone(), auth));

    Router::new()
        .merge(protected_routes_with_auth)
        .merge(unprotected_routes)
        .with_state(app_state)
}
