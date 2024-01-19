use axum::{
    body::StreamBody, extract::Path, headers::UserAgent, http::StatusCode, response::IntoResponse,
    routing::get, Router, TypedHeader,
};
use zstd::encode_all;

use crate::{
    repository::Repository,
    rpm::{index::get_repomd_index, package::RPMPackage},
};

use super::index::{get_primary_index, get_filelists_index, get_other_index};

async fn index(
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, StatusCode> {
    let repo = Repository::from_github(owner, repo).await;
    let Some(package) = repo.select_package_rpm(agent.as_str()) else {
        return Err(StatusCode::NOT_FOUND);
    };

    package.download().await.unwrap();

    let packages = vec![RPMPackage::from_package(package).unwrap()];

    match file.as_str() {
        "repomd.xml" => Ok(get_repomd_index(&packages).into_bytes()),
        "primary.xml.zst" => Ok(encode_all(get_primary_index(&packages).as_bytes(), 0).unwrap()),
        "filelists.xml.zst" => Ok(encode_all(get_filelists_index(&packages).as_bytes(), 0).unwrap()),
        "other.xml.zst" => Ok(encode_all(get_other_index(&packages).as_bytes(), 0).unwrap()),
        _ => Err(StatusCode::NOT_FOUND)
    }
}

async fn package(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");
    let res = reqwest::get(url).await.unwrap();
    let stream = res.bytes_stream();
    let stream = StreamBody::new(stream);

    stream
}

pub fn rpm_routes() -> Router {
    Router::new()
        .route("/github/:owner/:repo/repodata/:file", get(index))
        .route("/github/:owner/:repo/package/:ver/:file", get(package))
}
