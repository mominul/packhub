use axum::{
    body::Body, extract::Path, http::StatusCode, response::IntoResponse, routing::get, Router,
};
use axum_extra::{headers::UserAgent, typed_header::TypedHeader};
use tracing::debug;

use crate::{
    apt::index::{gzip_compression, AptIndices},
    repository::Repository,
};

async fn release_file(
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, StatusCode> {
    let repo = Repository::from_github(owner, repo).await;

    let package = repo.select_package_ubuntu(agent.as_str());

    debug!("Package selected {:?}", package);

    package.download().await.unwrap();

    // debug!("Downloaded package length {}", data.len());

    let packages = vec![package];

    let index = AptIndices::new(packages).unwrap();

    Ok(index.get_release_index())
}

async fn packages_file(
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, StatusCode> {
    let repo = Repository::from_github(owner, repo).await;

    let package = repo.select_package_ubuntu(agent.as_str());

    package.download().await.unwrap();

    let packages = vec![package];

    let index = AptIndices::new(packages).unwrap();

    match file.as_str() {
        "Packages" => Ok(index.get_package_index().as_bytes().to_owned()),
        "Packages.gz" => Ok(gzip_compression(index.get_package_index().as_bytes())),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn empty_packages_file(
    Path((_, _, file)): Path<(String, String, String)>,
) -> Result<Vec<u8>, StatusCode> {
    match file.as_str() {
        "Packages" => Ok(Vec::new()),
        "Packages.gz" => Ok(gzip_compression(&Vec::new())),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn pool(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    debug!("Pool request {ver} {file}");

    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");

    let res = reqwest::get(url).await.unwrap();

    let stream = res.bytes_stream();

    let stream = Body::from_stream(stream);

    stream
}

pub fn apt_routes() -> Router {
    Router::new()
        .route(
            "/github/:owner/:repo/dists/stable/Release",
            get(release_file),
        )
        .route(
            "/github/:owner/:repo/dists/stable/main/binary-amd64/:index",
            get(packages_file),
        )
        .route(
            "/github/:owner/:repo/dists/stable/main/binary-all/:index",
            get(empty_packages_file),
        )
        .route("/github/:owner/:repo/pool/stable/:ver/:file", get(pool))
}
