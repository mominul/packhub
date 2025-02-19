use std::{fs::File, sync::LazyLock, time::Duration};

use axum::{
    body::{Body, HttpBody}, http::{Response, StatusCode}, response::IntoResponse, routing::get, Router
};
use mongodb::Client;
use tokio::fs::read_to_string;
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
        .nest("/keys", pgp::keys())
}

async fn index() -> impl IntoResponse {
    match read_to_string("pages/index.html").await {
        Ok(response) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html; charset=utf-8")  // Explicit charset
            .body(Body::from(response))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Error loading page"))
            .unwrap(),
    }
}

pub fn app(client: Client) -> Router {
    Router::new()
        .route("/", get(index))
        .nest("/v1", v1())
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
