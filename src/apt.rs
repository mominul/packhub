use axum::{
    body::StreamBody, extract::Path, headers::UserAgent, response::IntoResponse, routing::get,
    Router, TypedHeader,
};

use crate::{repository::Repository, deb::DebAnalyzer};

async fn release_file(
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> String {
    let repo = Repository::from_github(owner, repo).await;

    let package = repo.select_package_ubuntu(agent.as_str());

    let data = reqwest::get(package.download_url()).await.unwrap().bytes().await.unwrap();

    let deb = DebAnalyzer::new(&data);
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
