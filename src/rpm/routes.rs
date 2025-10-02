use anyhow::{Context, anyhow};
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use axum_extra::{headers::UserAgent, typed_header::TypedHeader};
use zstd::encode_all;

use crate::{
    REQWEST,
    error::AppError,
    repository::Repository,
    rpm::{index::get_repomd_index, package::RPMPackage},
    state::AppState,
    utils::ReleaseChannel,
};

use super::index::{get_filelists_index, get_other_index, get_primary_index};

async fn handle_repo_index(
    state: &AppState,
    owner: &str,
    repo: &str,
    file: &str,
    channel: &ReleaseChannel,
    agent: &UserAgent,
) -> Result<Vec<u8>, AppError> {
    let mut repo = Repository::from_github(owner, repo, &channel, &state).await;
    let packages: Vec<RPMPackage> = repo
        .select_package_rpm(agent.as_str())
        .await?
        .into_iter()
        .map(|p| {
            RPMPackage::from_package(&p).context(format!(
                "Error while parsing package into RPMPackage: {p:?}"
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;

    repo.save_package_metadata().await;

    match file {
        "repomd.xml" => Ok(get_repomd_index(&packages).into_bytes()),
        "repomd.xml.asc" => {
            let metadata = get_repomd_index(&packages);
            let signature = state.detached_sign_metadata(&metadata)?;
            Ok(signature)
        }
        "repomd.xml.key" => Ok(state.armored_public_key()),
        "primary.xml.zst" => Ok(encode_all(get_primary_index(&packages).as_bytes(), 0)?),
        "filelists.xml.zst" => Ok(encode_all(get_filelists_index(&packages).as_bytes(), 0)?),
        "other.xml.zst" => Ok(encode_all(get_other_index(&packages).as_bytes(), 0)?),
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

async fn handle_repo_package(
    owner: &str,
    repo: &str,
    ver: &str,
    file: &str,
) -> Result<impl IntoResponse + use<>, AppError> {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");
    let res = REQWEST
        .get(url)
        .send()
        .await
        .context("Error occurred while proxying package")?;
    let stream = res.bytes_stream();
    let stream = Body::from_stream(stream);

    Ok(stream)
}

#[tracing::instrument(name = "RPM Index V1", skip_all, fields(agent = agent.as_str()))]
async fn index_v1(
    State(state): State<AppState>,
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, AppError> {
    handle_repo_index(
        &state,
        &owner,
        &repo,
        &file,
        &ReleaseChannel::Stable,
        &agent,
    )
    .await
}

#[tracing::instrument(name = "RPM Index V2", skip_all, fields(agent = agent.as_str()))]
async fn index_v2(
    State(state): State<AppState>,
    Path((owner, repo, channel, file)): Path<(String, String, ReleaseChannel, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, AppError> {
    handle_repo_index(&state, &owner, &repo, &file, &channel, &agent).await
}

#[tracing::instrument(name = "RPM Package proxy", skip_all)]
async fn package_v1(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    handle_repo_package(&owner, &repo, &ver, &file).await
}

#[tracing::instrument(name = "RPM Package proxy V2", skip_all)]
async fn package_v2(
    Path((owner, repo, _channel, ver, file)): Path<(
        String,
        String,
        ReleaseChannel,
        String,
        String,
    )>,
) -> Result<impl IntoResponse, AppError> {
    handle_repo_package(&owner, &repo, &ver, &file).await
}

pub fn rpm_routes_v1() -> Router<AppState> {
    Router::new()
        .route("/github/{owner}/{repo}/repodata/{file}", get(index_v1))
        .route(
            "/github/{owner}/{repo}/package/{ver}/{file}",
            get(package_v1),
        )
}

pub fn rpm_routes_v2() -> Router<AppState> {
    Router::new()
        .route(
            "/github/{owner}/{repo}/{channel}/repodata/{file}",
            get(index_v2),
        )
        .route(
            "/github/{owner}/{repo}/{channel}/package/{ver}/{file}",
            get(package_v2),
        )
}
