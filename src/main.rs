use std::{env::args, net::SocketAddr};

use mongodb::Client;
use tracing::{info, Level};
use tracing_subscriber::{filter::Targets, prelude::*};
use dotenvy::{dotenv, var};

use packhub::{app, pgp::generate_and_save_keys};

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
    
    if dotenv().is_err() {
        info!("No .env file found");
    }

    if args().len() > 1 {
        let arg = args().nth(1).unwrap();
        if arg == "--generate-keys" {
            generate_and_save_keys().unwrap();
        }
    }

    let uri = format!("mongodb://{}:{}@localhost:27017", var("PACKHUB_DB_USER").unwrap(), var("PACKHUB_DB_PASSWORD").unwrap());

    let client = Client::with_uri_str(uri)
        .await
        .unwrap();

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();

    info!("listening on {}", addr);

    // run it
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app(client)).await.unwrap();
}
