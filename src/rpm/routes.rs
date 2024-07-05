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
use zstd::encode_all;

use crate::{
    pgp::{detached_sign_metadata, load_secret_key_from_file},
    repository::Repository,
    rpm::{index::get_repomd_index, package::RPMPackage},
};

use super::index::{get_filelists_index, get_other_index, get_primary_index};

async fn index(
    State(client): State<Client>,
    Path((owner, repo, file)): Path<(String, String, String)>,
    TypedHeader(agent): TypedHeader<UserAgent>,
) -> Result<Vec<u8>, StatusCode> {
    let repo = Repository::from_github(owner, repo, client).await;
    let packages: Vec<RPMPackage> = repo
        .select_package_rpm(agent.as_str())
        .await
        .unwrap()
        .into_iter()
        .map(|p| RPMPackage::from_package(&p).unwrap())
        .collect();

    match file.as_str() {
        "repomd.xml" => Ok(get_repomd_index(&packages).into_bytes()),
        "repomd.xml.asc" => {
            let metadata = get_repomd_index(&packages);
            let secret_key = load_secret_key_from_file().unwrap();
            let signature = detached_sign_metadata("repomd.xml", &metadata, &secret_key)
                .unwrap()
                .into_bytes();
            Ok(signature)
        }
        "primary.xml.zst" => Ok(encode_all(get_primary_index(&packages).as_bytes(), 0).unwrap()),
        "filelists.xml.zst" => {
            Ok(encode_all(get_filelists_index(&packages).as_bytes(), 0).unwrap())
        }
        "other.xml.zst" => Ok(encode_all(get_other_index(&packages).as_bytes(), 0).unwrap()),
        _ => Err(StatusCode::NOT_FOUND),
    }
}

async fn package(
    Path((owner, repo, ver, file)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    let url = format!("https://github.com/{owner}/{repo}/releases/download/{ver}/{file}");
    let res = reqwest::get(url).await.unwrap();
    let stream = res.bytes_stream();
    let stream = Body::from_stream(stream);

    stream
}

pub fn rpm_routes() -> Router<Client> {
    Router::new()
        .route("/github/:owner/:repo/repodata/:file", get(index))
        .route("/github/:owner/:repo/package/:ver/:file", get(package))
}
