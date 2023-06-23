use std::fmt::Write;

use axum::{
    body::StreamBody, extract::Path, headers::UserAgent, response::IntoResponse, routing::get,
    Router, TypedHeader,
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

mod detect;
mod platform;
mod repository;
mod selector;

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
            // "{} - {} - {:?}\n{}\n{}\n\n",
            "{},\n",
            asset.name,
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

async fn apt_pool(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");

    let res = reqwest::get(url).await.unwrap();

    let stream = res.bytes_stream();

    let stream = StreamBody::new(stream);

    stream
}

pub fn app() -> Router {
    Router::new()
        .route("/apt/github/:owner/:repo", get(handler))
        .route(
            "/apt/github/:owner/:repo/pool/stable/:ver/:file",
            get(apt_pool),
        )
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        )
}
