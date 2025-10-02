use anyhow::anyhow;
use axum::{
    Router,
    extract::{Path, Query},
    routing::get,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    state::AppState,
    utils::{AppVersion, ReleaseChannel},
};

mod apt;
mod rpm;

#[derive(Deserialize)]
struct ScriptParams {
    prerelease: Option<bool>,
    ver: Option<AppVersion>,
}

async fn script_handler(
    Path((distro, owner, repo)): Path<(String, String, String)>,
    Query(params): Query<ScriptParams>,
) -> Result<String, AppError> {
    let ver = params.ver.unwrap_or(AppVersion::V2);
    let channel = if let Some(prerelease) = params.prerelease
        && prerelease
    {
        ReleaseChannel::Unstable
    } else {
        ReleaseChannel::Stable
    };

    match distro.as_str() {
        "ubuntu" | "debian" => Ok(apt::generate_apt_script(&distro, &owner, &repo, &channel)),
        "yum" => Ok(rpm::generate_rpm_script(
            &owner,
            &repo,
            "yum.repos.d",
            &ver,
            &channel,
        )),
        "zypp" => Ok(rpm::generate_rpm_script(
            &owner,
            &repo,
            "zypp/repos.d",
            &ver,
            &channel,
        )),
        _ => Err(anyhow!("Script Generation: Unsupported distro: {}", distro).into()),
    }
}

pub fn script_routes() -> Router<AppState> {
    Router::new().route("/{distro}/github/{owner}/{repo}", get(script_handler))
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;
    use insta::assert_snapshot;

    use super::*;

    #[tokio::test]
    async fn test_script_apt_endpoint() {
        dotenvy::dotenv().unwrap();
        let state = AppState::initialize_for_test().await;
        let server = TestServer::new(script_routes().with_state(state)).unwrap();

        let stable = server
            .get("/ubuntu/github/OpenBangla/OpenBangla-Keyboard")
            .await
            .text();

        assert_snapshot!(stable);

        let unstable = server
            .get("/ubuntu/github/OpenBangla/OpenBangla-Keyboard?prerelease=true")
            .await
            .text();

        assert_snapshot!(unstable);
    }

    #[tokio::test]
    async fn test_script_rpm_endpoint() {
        dotenvy::dotenv().unwrap();
        let state = AppState::initialize_for_test().await;
        let server = TestServer::new(script_routes().with_state(state)).unwrap();

        let yum_stable = server
            .get("/yum/github/OpenBangla/OpenBangla-Keyboard")
            .await
            .text();

        assert_snapshot!(yum_stable);

        let zypp_stable = server
            .get("/zypp/github/OpenBangla/OpenBangla-Keyboard")
            .await
            .text();

        assert_snapshot!(zypp_stable);

        let yum_unstable = server
            .get("/yum/github/OpenBangla/OpenBangla-Keyboard?prerelease=true")
            .await
            .text();

        assert_snapshot!(yum_unstable);

        let zypp_unstable = server
            .get("/zypp/github/OpenBangla/OpenBangla-Keyboard?prerelease=true")
            .await
            .text();

        assert_snapshot!(zypp_unstable);

        let yum_unstable_v2 = server
            .get("/yum/github/OpenBangla/OpenBangla-Keyboard?prerelease=true&ver=v2")
            .await
            .text();

        assert_eq!(yum_unstable, yum_unstable_v2);

        let zypp_unstable_v2 = server
            .get("/zypp/github/OpenBangla/OpenBangla-Keyboard?prerelease=true&ver=v2")
            .await
            .text();

        assert_eq!(zypp_unstable, zypp_unstable_v2);

        let yum_v1 = server
            .get("/yum/github/OpenBangla/OpenBangla-Keyboard?ver=v1")
            .await
            .text();

        assert_snapshot!(yum_v1);

        let zypp_v1 = server
            .get("/zypp/github/OpenBangla/OpenBangla-Keyboard?ver=v1")
            .await
            .text();

        assert_snapshot!(zypp_v1);
    }
}
