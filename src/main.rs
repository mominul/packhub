use packhub::app;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("listening on {}", addr);

    // run it
    axum::Server::bind(&addr)
        .serve(app().into_make_service())
        .await
        .unwrap();
}
