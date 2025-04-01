use std::{sync::LazyLock, time::Duration};

use axum::{
    Router,
    body::{Body, HttpBody},
    http::Response,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{Span, debug};

use crate::state::AppState;

mod apt;
mod db;
mod detect;
mod error;
mod package;
pub mod pgp;
mod platform;
mod repository;
mod rpm;
mod script;
mod selector;
pub mod state;
mod utils;

static REQWEST: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .unwrap()
});

fn v1() -> Router<AppState> {
    Router::new()
        .nest("/apt", apt::apt_routes())
        .nest("/rpm", rpm::rpm_routes())
        .nest("/keys", pgp::keys())
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route_service("/", ServeFile::new("pages/index.html"))
        .nest("/v1", v1())
        .nest("/sh", script::script_routes())
        .nest_service("/assets", ServeDir::new("pages/assets"))
        .with_state(state)
        .layer(TraceLayer::new_for_http().on_response(
            |response: &Response<Body>, latency: Duration, _: &Span| {
                let size = response.body().size_hint().upper().unwrap_or(0);
                let status = response.status();
                debug!(size=size,latency=?latency,status=%status, "finished processing request");
            },
        ))
}
