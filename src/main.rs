use std::net::SocketAddr;
use axum::{Router, routing::get, extract::Path, TypedHeader, headers::UserAgent};
use tracing::{info, instrument};

#[instrument]
async fn handler(Path(proj): Path<String>, TypedHeader(agent): TypedHeader<UserAgent>) -> String {
    info!("");
    format!("Project name: {proj}\n\nUser Agent: {agent}")
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let app = Router::new().route("/apt/:proj", get(handler));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
