use axum::{routing::get, Router, http::StatusCode};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

mod apt;
mod package;
mod platform;
mod repository;
mod rpm;
mod selector;
mod utils;

pub fn app() -> Router {
    Router::new()
        .nest("/apt", apt::apt_routes())
        .nest("/rpm", rpm::rpm_routes())
        .route("/", get(|| async { StatusCode::OK }))
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
}
