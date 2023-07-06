use axum::{
    body::StreamBody, extract::Path, headers::UserAgent, response::IntoResponse, routing::get,
    Router, TypedHeader,
};

use crate::repository::Repository;

async fn release_file(
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> String {
    let repo = Repository::from_github(owner, repo).await;
    todo!()
}

async fn pool(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");

    let res = reqwest::get(url).await.unwrap();

    let stream = res.bytes_stream();

    let stream = StreamBody::new(stream);

    stream
}

pub fn apt_routes() -> Router {
    Router::new()
        .route(
            "/github/:owner/:repo/dists/stable/Release",
            get(release_file),
        )
        .route("/github/:owner/:repo/pool/stable/:ver/:file", get(pool))
}
