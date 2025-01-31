use std::{sync::LazyLock, time::Duration};

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

static REQWEST: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .unwrap()
});

fn v1() -> Router<Client> {
    Router::new()
        .nest("/apt", apt::apt_routes())
        .nest("/rpm", rpm::rpm_routes())
}

pub fn app(client: Client) -> Router {
    Router::new()
        .route("/", get(|| async { StatusCode::OK }))
        .nest("/v1", v1())
        .nest("/keys", pgp::keys())
        .nest("/sh", script::script_routes())
        .with_state(client)
        .layer(TraceLayer::new_for_http().on_response(
            |response: &Response<Body>, latency: Duration, _: &Span| {
                let size = response.body().size_hint().upper().unwrap_or(0);
                let status = response.status();
                debug!(size=size,latency=?latency,status=%status, "finished processing request");
            },
        ))
}
