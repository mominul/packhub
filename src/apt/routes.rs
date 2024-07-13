use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::{headers::UserAgent, typed_header::TypedHeader};
use mongodb::Client;
use tracing::debug;

use crate::{
    apt::index::{gzip_compression, AptIndices},
    pgp::{clearsign_metadata, detached_sign_metadata, load_secret_key_from_file},
    repository::Repository,
};

#[tracing::instrument(name = "Debian Clear-signed Release File", skip(owner, repo, client))]
async fn in_release_file(
    State(client): State<Client>,
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, StatusCode> {
    let mut repo = Repository::from_github(owner, repo, client).await;

    let packages = repo.select_package_ubuntu(agent.as_str()).await.unwrap();

    let index = AptIndices::new(&packages).unwrap();

    repo.save_package_metadata().await;

    let release_file = index.get_release_index();

    let secret_key = load_secret_key_from_file().unwrap();

    let signed_release_file = clearsign_metadata(&release_file, &secret_key).unwrap();

    Ok(signed_release_file)
}

#[tracing::instrument(name = "Debian Release File", skip(owner, repo, client))]
async fn release_file(
    State(client): State<Client>,
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, StatusCode> {
    let mut repo = Repository::from_github(owner, repo, client).await;

    let packages = repo.select_package_ubuntu(agent.as_str()).await.unwrap();

    let index = AptIndices::new(&packages).unwrap();

    repo.save_package_metadata().await;

    Ok(index.get_release_index())
}

#[tracing::instrument(name = "Debian Signed Release File", skip(owner, repo, client))]
async fn signed_release_file(
    State(client): State<Client>,
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, StatusCode> {
    let mut repo = Repository::from_github(owner, repo, client).await;

    let packages = repo.select_package_ubuntu(agent.as_str()).await.unwrap();

    let index = AptIndices::new(&packages).unwrap();

    repo.save_package_metadata().await;

    let release_file = index.get_release_index();

    let secret_key = load_secret_key_from_file().unwrap();

    let signed_release_file =
        detached_sign_metadata("Release", &release_file, &secret_key).unwrap();

    Ok(signed_release_file)
}

#[tracing::instrument(name = "Debian Package metadata file", skip(owner, repo, file, client))]
async fn packages_file(
    State(client): State<Client>,
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, StatusCode> {
    let mut repo = Repository::from_github(owner, repo, client).await;

    let packages = repo.select_package_ubuntu(agent.as_str()).await.unwrap();

    let index = AptIndices::new(&packages).unwrap();

    repo.save_package_metadata().await;

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

#[tracing::instrument(name = "Debian Package download", skip_all)]
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

pub fn apt_routes() -> Router<Client> {
    Router::new()
        .route(
            "/github/:owner/:repo/dists/stable/Release",
            get(release_file),
        )
        .route(
            "/github/:owner/:repo/dists/stable/Release.gpg",
            get(signed_release_file),
        )
        .route(
            "/github/:owner/:repo/dists/stable/InRelease",
            get(in_release_file),
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
