use anyhow::{anyhow, Context};
use axum::{
    body::Body,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::{headers::UserAgent, typed_header::TypedHeader};
use zstd::encode_all;

use crate::{
    error::AppError,
    repository::Repository,
    rpm::{index::get_repomd_index, package::RPMPackage},
    state::AppState,
    REQWEST,
};

use super::index::{get_filelists_index, get_other_index, get_primary_index};

#[tracing::instrument(name = "RPM Index", skip_all, fields(agent = agent.as_str()))]
async fn index(
    State(state): State<AppState>,
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, AppError> {
    let mut repo = Repository::from_github(owner, repo, &state).await;
    let packages: Vec<RPMPackage> = repo
        .select_package_rpm(agent.as_str())
        .await?
        .into_iter()
        .map(|p| RPMPackage::from_package(&p).unwrap())
        .collect();

    repo.save_package_metadata().await;

    match file.as_str() {
        "repomd.xml" => Ok(get_repomd_index(&packages).into_bytes()),
        "repomd.xml.asc" => {
            let metadata = get_repomd_index(&packages);
            let signature = state.detached_sign_metadata(&metadata)?.into_bytes();
            Ok(signature)
        }
        "repomd.xml.key" => {
            let public_key = std::fs::read_to_string("packhub.asc").unwrap();
            Ok(public_key.into_bytes())
        }
        "primary.xml.zst" => Ok(encode_all(get_primary_index(&packages).as_bytes(), 0)?),
        "filelists.xml.zst" => Ok(encode_all(get_filelists_index(&packages).as_bytes(), 0)?),
        "other.xml.zst" => Ok(encode_all(get_other_index(&packages).as_bytes(), 0)?),
        file => Err(anyhow!("Unknown file requested: {file}").into()),
    }
}

#[tracing::instrument(name = "RPM Package proxy", skip_all)]
async fn package(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
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

pub fn rpm_routes() -> Router<AppState> {
    Router::new()
        .route("/github/{owner}/{repo}/repodata/{file}", get(index))
        .route("/github/{owner}/{repo}/package/{ver}/{file}", get(package))
}
