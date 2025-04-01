use anyhow::{Context, anyhow};
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use axum_extra::{headers::UserAgent, typed_header::TypedHeader};

use crate::{
    REQWEST,
    apt::index::{AptIndices, gzip_compression},
    error::AppError,
    repository::Repository,
    state::AppState,
};

#[tracing::instrument(name = "Debian Release File", skip_all, fields(agent = agent.as_str()))]
async fn release_index(
    State(state): State<AppState>,
    Path((distro, owner, repo, file)): Path<(String, String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<String, AppError> {
    let mut repo = Repository::from_github(owner, repo, &state).await;
    let packages = repo.select_package_apt(&distro, agent.as_str()).await?;

    let index = AptIndices::new(&packages)?;
    repo.save_package_metadata().await;

    let release_file = index.get_release_index();

    match file.as_str() {
        "Release" => Ok(release_file),
        "Release.gpg" => {
            let signed_release_file = state.detached_sign_metadata(&release_file)?;
            Ok(signed_release_file)
        }
        "InRelease" => {
            let signed_release_file = state.clearsign_metadata(&release_file)?;

            Ok(signed_release_file)
        }
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

#[tracing::instrument(name = "Debian Package metadata file", skip_all, fields(agent = agent.as_str()))]
async fn packages_file(
    State(state): State<AppState>,
    Path((distro, owner, repo, file)): Path<(String, String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, AppError> {
    let mut repo = Repository::from_github(owner, repo, &state).await;
    let packages = repo.select_package_apt(&distro, agent.as_str()).await?;

    let index = AptIndices::new(&packages)?;
    repo.save_package_metadata().await;

    match file.as_str() {
        "Packages" => Ok(index.get_package_index().as_bytes().to_owned()),
        "Packages.gz" => Ok(gzip_compression(index.get_package_index().as_bytes())),
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

async fn empty_packages_file(
    Path((_, _, _, file)): Path<(String, String, String, String)>,
) -> Result<Vec<u8>, AppError> {
    match file.as_str() {
        "Packages" => Ok(Vec::new()),
        "Packages.gz" => Ok(gzip_compression(&Vec::new())),
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

#[tracing::instrument(name = "Debian Package proxy", skip_all)]
async fn pool(
    Path((_, owner, repo, ver, file)): Path<(String, String, String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");
    tracing::trace!("Proxying package from: {}", url);
    let res = REQWEST
        .get(url)
        .send()
        .await
        .context("Error occurred while proxying package")?;
    tracing::trace!("Proxying package respone: {}", res.status());
    let stream = Body::from_stream(res.bytes_stream());

    Ok(stream)
}

pub fn apt_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/{distro}/github/{owner}/{repo}/dists/stable/{file}",
            get(release_index),
        )
        .route(
            "/{distro}/github/{owner}/{repo}/dists/stable/main/binary-amd64/{index}",
            get(packages_file),
        )
        .route(
            "/{distro}/github/{owner}/{repo}/dists/stable/main/binary-all/{index}",
            get(empty_packages_file),
        )
        .route(
            "/{distro}/github/{owner}/{repo}/pool/stable/{ver}/{file}",
            get(pool),
        )
}
