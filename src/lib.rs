use std::time::Duration;

use axum::{
    body::{Body, HttpBody},
    http::{Response, StatusCode},
    routing::get,
    Router,
};
use mongodb::Client;
use tower_http::trace::TraceLayer;
use tracing::{debug, Span};

mod apt;
mod db;
mod error;
mod package;
pub mod pgp;
mod platform;
mod repository;
mod rpm;
mod script;
mod selector;
mod utils;

pub fn app(client: Client) -> Router {
    Router::new()
        .route("/", get(|| async { StatusCode::OK }))
        .nest("/apt", apt::apt_routes())
        .nest("/rpm", rpm::rpm_routes())
        .nest("/keys", pgp::keys())
        .nest("/sh", script::script_routes())
        .with_state(client)
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
