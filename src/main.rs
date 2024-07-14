use mongodb::Client;
use packhub::{app, pgp::generate_and_save_keys};
use std::{env::args, net::SocketAddr};
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, prelude::*};

#[tokio::main]
async fn main() {
    let filter = Targets::new()
        .with_target("tower_http", Level::TRACE)
        .with_target("packhub", Level::TRACE)
        .with_default(Level::INFO);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    if args().len() > 1 {
        let arg = args().nth(1).unwrap();
        if arg == "--generate-keys" {
            generate_and_save_keys().unwrap();
        }
    }

    let client = Client::with_uri_str("mongodb://root:pass@localhost:27017")
        .await
        .unwrap();

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    info!("listening on {}", addr);

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app(client)).await.unwrap();
}
