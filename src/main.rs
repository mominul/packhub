use packhub::app;
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, prelude::*};

#[tokio::main]
async fn main() {
    let filter = Targets::new()
        .with_target("tower_http::trace::on_response", Level::TRACE)
        .with_target("tower_http::trace::on_request", Level::TRACE)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    info!("listening on {}", addr);

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app()).await.unwrap();
}
