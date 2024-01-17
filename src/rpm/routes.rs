use axum::{
    body::StreamBody, extract::Path, headers::UserAgent, http::StatusCode, response::IntoResponse,
    routing::get, Router, TypedHeader,
};

use crate::{
    repository::Repository,
    rpm::{index::get_repomd_index, package::RPMPackage},
};

async fn repomd(
    Path((owner, repo)): Path<(String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, StatusCode> {
    let repo = Repository::from_github(owner, repo).await;
    let Some(package) = repo.select_package_rpm(agent.as_str()) else {
        return Err(StatusCode::NOT_FOUND);
    };

    package.download().await.unwrap();

    let package = vec![RPMPackage::from_package(package).unwrap()];
    let index = get_repomd_index(&package).into_bytes();

    Ok(index)
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
        .route("/github/:owner/:repo/repodata/repomd.xml", get(repomd))
        .route("/github/:owner/:repo/package/:ver/:file", get(package))
}
