use axum::Router;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

mod apt;
mod rpm;
mod package;
mod platform;
mod repository;
mod selector;
mod utils;

pub fn app() -> Router {
    Router::new()
        .nest("/apt", apt::apt_routes())
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
}
