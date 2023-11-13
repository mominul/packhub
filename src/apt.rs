use axum::{
    body::StreamBody, extract::Path, headers::UserAgent, response::IntoResponse, routing::get,
    Router, TypedHeader,
};
use tracing::debug;

use crate::{repository::Repository, index::{AptIndices, gzip_compression}};

async fn release_file(
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> String {
    let repo = Repository::from_github(owner, repo).await;

    let package = repo.select_package_ubuntu(agent.as_str());

    debug!("Package selected {:?}", package);

    let data = reqwest::get(package.download_url()).await.unwrap().bytes().await.unwrap();

    debug!("Downloaded package length {}", data.len());

    let index = AptIndices::new(package, &data);
    
    index.get_release_index()
}

async fn packages_file(Path((owner, repo, file)): Path<(String, String, String)>, TypedHeader(agent): TypedHeader<UserAgent>) -> Vec<u8> {
    let repo = Repository::from_github(owner, repo).await;

    let package = repo.select_package_ubuntu(agent.as_str());

    let data = reqwest::get(package.download_url()).await.unwrap().bytes().await.unwrap();

    let index = AptIndices::new(package, &data);
    
    match file.as_str() {
        "Packages" => index.get_package_index().as_bytes().to_owned(),
        "Packages.gz" => gzip_compression(index.get_package_index().as_bytes()),
        _ => panic!()
    }
}

async fn empty_packages_file(Path((owner, repo, file)): Path<(String, String, String)>, TypedHeader(agent): TypedHeader<UserAgent>) -> Vec<u8> {
    match file.as_str() {
        "Packages" => Vec::new(),
        "Packages.gz" => gzip_compression(&Vec::new()),
        _ => panic!()
    }
}

async fn pool(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    debug!("Pool request {ver} {file}");
    
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
        .route("/github/:owner/:repo/dists/stable/main/binary-amd64/:index", get(packages_file))
        .route("/github/:owner/:repo/dists/stable/main/binary-all/:index", get(empty_packages_file))
        .route("/github/:owner/:repo/pool/stable/:ver/:file", get(pool))
}
