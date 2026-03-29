use askama::Template;
use axum::{Router, extract::State, response::Html, routing::get};

use crate::app_state::AppState;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    title: &'a str,
}

#[derive(Template)]
#[template(path = "music.html")]
struct MusicTemplate<'a> {
    heading: &'a str,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(home))
        .route("/home", get(home))
        .route("/music", get(music))
        .route("/library", get(library))
        .route("/stats", get(stats))
        .route("/search", get(search))
}

pub async fn home(State(_state): State<AppState>) -> Html<String> {
    Html(HomeTemplate { title: "synthwaves.fm (Rust/Axum)" }.render().unwrap_or_else(|_| "<h1>synthwaves.fm</h1>".to_string()))
}

pub async fn music() -> Html<String> {
    Html(MusicTemplate { heading: "Music" }.render().unwrap_or_else(|_| "<h1>Music</h1>".to_string()))
}

pub async fn library() -> Html<String> {
    Html("<h1>Library</h1>".to_string())
}

pub async fn stats() -> Html<String> {
    Html("<h1>Stats</h1>".to_string())
}

pub async fn search() -> Html<String> {
    Html("<h1>Search</h1>".to_string())
}
