use std::fmt::Write;

use axum::{extract::Path, headers::UserAgent, routing::get, Router, TypedHeader};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

mod apt;
mod rpm;
mod detect;
mod platform;
mod repository;
mod selector;
mod utils;

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
    let ver = rel.tag_name.clone();
    write!(
        &mut response,
        "Project name: {repo}\nLatest Release: {name}\nVersion: {ver}\nAssets:\n"
    )
    .unwrap();

    for asset in rel.assets {
        write!(
            &mut response,
            // "{} - {} - {:?}\n{}\n{}\n\n",
            "{} - {},\n",
            asset.name,
            asset.browser_download_url.as_str(),
            // asset.browser_download_url.as_str(),
            // asset.updated_at.cmp(&asset.created_at),
            // asset.created_at,
            // asset.updated_at
        )
        .unwrap();
    }
    write!(&mut response, "\n\nUser Agent: {agent}").unwrap();

    response
}

pub fn app() -> Router {
    Router::new()
        .route("/test/github/:owner/:repo", get(handler))
        .nest("/apt", apt::apt_routes())
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
}
