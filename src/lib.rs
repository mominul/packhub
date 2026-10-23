use std::{path::PathBuf, sync::LazyLock, time::Duration};

use axum::{
    Router,
    body::{Body, HttpBody},
    extract::{Extension, Path},
    http::{Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{Span, debug};

use crate::state::AppState;

mod apt;
mod metadata;
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
pub mod cron;

static REQWEST: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .user_agent("PackHub/1.0 (https://packhub.dev)")
        .build()
        .unwrap()
});

fn v1() -> Router<AppState> {
    Router::new()
        .nest("/apt", apt::apt_routes())
        .nest("/rpm", rpm::rpm_routes_v1())
        .nest("/keys", pgp::keys())
}

fn v2() -> Router<AppState> {
    Router::new().nest("/rpm", rpm::rpm_routes_v2())
}

const MAX_ACME_TOKEN_FILE_SIZE: u64 = 4096;

async fn acme_challenge(
    Extension(webroot): Extension<PathBuf>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let valid = !token.is_empty()
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');

    if !valid {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = webroot.join(token);
    let Ok(metadata) = tokio::fs::symlink_metadata(&path).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if !metadata.file_type().is_file() || metadata.len() > MAX_ACME_TOKEN_FILE_SIZE {
        return StatusCode::NOT_FOUND.into_response();
    }

    match tokio::fs::read(&path).await {
        Ok(body) => ([(header::CONTENT_TYPE, "text/plain")], body).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn app(state: AppState) -> Router {
    let mut router = Router::new()
        .route_service("/", ServeFile::new("pages/index.html"))
        .nest("/v1", v1())
        .nest("/v2", v2())
        .nest("/sh", script::script_routes())
        .nest_service("/assets", ServeDir::new("pages/assets"));

    if let Ok(webroot) = std::env::var("PACKHUB_ACME_WEBROOT") {
        let webroot = webroot.trim();
        if !webroot.is_empty() {
            router = router
                .route("/.well-known/acme-challenge/{token}", get(acme_challenge))
                .layer(Extension(PathBuf::from(webroot)));
        }
    }

    router
        .with_state(state)
        .layer(TraceLayer::new_for_http().on_response(
            |response: &Response<Body>, latency: Duration, _: &Span| {
                let size = response.body().size_hint().upper().unwrap_or(0);
                let status = response.status();
                debug!(size=size,latency=?latency,status=%status, "finished processing request");
            },
        ))
}
