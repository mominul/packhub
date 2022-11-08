use axum::{extract::Path, headers::UserAgent, routing::get, Router, TypedHeader};
use std::fmt::Write;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

async fn handler(
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> String {
    let mut response = String::new();

    let octocrab = octocrab::instance();
    let rel = octocrab
        .repos(&owner, &repo)
        .releases()
        .get_latest()
        .await
        .unwrap();
        
    let name = rel.name.clone().unwrap();
    write!(
        &mut response,
        "Project name: {repo}\nLatest Release: {name}\nAssets:\n"
    )
    .unwrap();

    for asset in rel.assets {
        write!(
            &mut response,
            "{} - {}\n",
            asset.name,
            asset.browser_download_url.as_str()
        )
        .unwrap();
    }
    write!(&mut response, "\n\nUser Agent: {agent}").unwrap();

    response
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/apt/github/:owner/:repo", get(handler))
        .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
