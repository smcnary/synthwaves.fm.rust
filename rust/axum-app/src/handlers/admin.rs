use axum::{Router, response::Html, routing::get};

use crate::app_state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/users", get(users))
        .route("/jobs", get(jobs))
}

pub async fn index() -> Html<&'static str> {
    Html("<h1>Admin</h1><p>Axum admin replacement scaffold.</p>")
}

pub async fn users() -> Html<&'static str> {
    Html("<h1>Admin Users</h1>")
}

pub async fn jobs() -> Html<&'static str> {
    Html("<h1>Admin Jobs</h1>")
}
