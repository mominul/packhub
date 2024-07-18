use anyhow::{anyhow, Context};
use axum::{
    body::Body,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::{headers::UserAgent, typed_header::TypedHeader};
use mongodb::Client;

use crate::{
    apt::index::{gzip_compression, AptIndices},
    error::AppError,
    pgp::{clearsign_metadata, detached_sign_metadata, load_secret_key_from_file},
    repository::Repository,
};

#[tracing::instrument(name = "Debian Clear-signed Release File", skip_all, fields(agent = agent.as_str()))]
async fn in_release_file(
    State(client): State<Client>,
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, AppError> {
    let mut repo = Repository::from_github(owner, repo, client).await;
    let packages = repo.select_package_ubuntu(agent.as_str()).await?;

    let index = AptIndices::new(&packages)?;
    repo.save_package_metadata().await;

    let release_file = index.get_release_index();
    let secret_key = load_secret_key_from_file()?;
    let signed_release_file = clearsign_metadata(&release_file, &secret_key)?;

    Ok(signed_release_file)
}

#[tracing::instrument(name = "Debian Release File", skip_all, fields(agent = agent.as_str()))]
async fn release_file(
    State(client): State<Client>,
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, AppError> {
    let mut repo = Repository::from_github(owner, repo, client).await;
    let packages = repo.select_package_ubuntu(agent.as_str()).await?;

    let index = AptIndices::new(&packages)?;
    repo.save_package_metadata().await;

    Ok(index.get_release_index())
}

#[tracing::instrument(name = "Debian Signed Release File", skip_all, fields(agent = agent.as_str()))]
async fn signed_release_file(
    State(client): State<Client>,
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, AppError> {
    let mut repo = Repository::from_github(owner, repo, client).await;
    let packages = repo.select_package_ubuntu(agent.as_str()).await?;

    let index = AptIndices::new(&packages)?;
    repo.save_package_metadata().await;

    let release_file = index.get_release_index();
    let secret_key = load_secret_key_from_file()?;
    let signed_release_file = detached_sign_metadata("Release", &release_file, &secret_key)?;

    Ok(signed_release_file)
}

#[tracing::instrument(name = "Debian Package metadata file", skip_all, fields(agent = agent.as_str()))]
async fn packages_file(
    State(client): State<Client>,
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, AppError> {
    let mut repo = Repository::from_github(owner, repo, client).await;
    let packages = repo.select_package_ubuntu(agent.as_str()).await?;

    let index = AptIndices::new(&packages)?;
    repo.save_package_metadata().await;

    match file.as_str() {
        "Packages" => Ok(index.get_package_index().as_bytes().to_owned()),
        "Packages.gz" => Ok(gzip_compression(index.get_package_index().as_bytes())),
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

async fn empty_packages_file(
    Path((_, _, file)): Path<(String, String, String)>,
) -> Result<Vec<u8>, AppError> {
    match file.as_str() {
        "Packages" => Ok(Vec::new()),
        "Packages.gz" => Ok(gzip_compression(&Vec::new())),
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

#[tracing::instrument(name = "Debian Package proxy", skip_all)]
async fn pool(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");
    let res = reqwest::get(url)
        .await
        .context("Error occurred while proxying package")?;
    let stream = Body::from_stream(res.bytes_stream());

    Ok(stream)
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
