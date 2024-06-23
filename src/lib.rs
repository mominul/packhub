use std::time::Duration;

use axum::{
    body::{Body, HttpBody},
    http::{Response, StatusCode},
    routing::get,
    Router,
};
use tower_http::trace::TraceLayer;
use tracing::{debug, Span};

mod apt;
mod package;
pub mod pgp;
mod platform;
mod repository;
mod rpm;
mod script;
mod selector;
mod utils;

async fn public_key() -> String {
    std::fs::read_to_string("packhub.asc").unwrap()
}

pub fn app() -> Router {
    Router::new()
        .nest("/apt", apt::apt_routes())
        .nest("/rpm", rpm::rpm_routes())
        .route("/", get(|| async { StatusCode::OK }))
        .route("/keys/packhub.asc", get(public_key))
        .nest("/sh", script::script_routes())
        .layer(
            TraceLayer::new_for_http().on_response(|response: &Response<Body>, latency: Duration, _: &Span| {
                debug!(size=response_size(response),latency=?latency,status=%response_status(response), "finished processing request");
            })
        )
}

fn response_size(response: &Response<Body>) -> u64 {
    response.body().size_hint().upper().unwrap_or(0)
}

fn response_status(response: &Response<Body>) -> StatusCode {
    response.status()
}
