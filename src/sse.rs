use std::{collections::HashSet, convert::Infallible};

use axum::{
    Router,
    extract::{Query, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
};
use futures_util::stream::unfold;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    package::Package,
    state::AppState,
    utils::{Dist, Digest, Type},
};

#[derive(Serialize)]
struct PackageEvent {
    file_name: String,
    package_type: String,
    distro: Option<String>,
    architecture: String,
    name: Option<String>,
}

#[derive(Serialize)]
struct LinkEvent {
    platform: String,
    label: String,
    url: String,
}

#[derive(Deserialize)]
struct SseQuery {
    owner: String,
    repo: String,
}

async fn sse_detect_handler(
    State(state): State<AppState>,
    Query(query): Query<SseQuery>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);

    let owner = query.owner;
    let repo = query.repo;

    tokio::spawn(async move {
        let release = match state
            .github()
            .repos(&owner, &repo)
            .releases()
            .get_latest()
            .await
        {
            Ok(release) => release,
            Err(e) => {
                error!("Failed to fetch latest release: {e}");
                let _ = tx
                    .send(
                        Event::default()
                            .event("error")
                            .json_data(serde_json::json!({ "message": format!("Failed to fetch latest release: {e}") }))
                            .unwrap(),
                    )
                    .await;
                return;
            }
        };

        let mut detected_platforms: HashSet<String> = HashSet::new();

        for asset in &release.assets {
            let digest = asset.digest.as_ref().and_then(|d| {
                d.strip_prefix("sha256:")
                    .map(|hash| Digest::Sha256(hash.to_string()))
            });

            let package = match Package::detect_package(
                &asset.name,
                release.tag_name.clone(),
                asset.browser_download_url.to_string(),
                digest,
                asset.updated_at,
            ) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let package_type = match package.ty() {
                Type::Deb => "deb",
                Type::Rpm => "rpm",
            };

            let distro_str = package.distribution().map(|d| match d {
                Dist::Ubuntu(v) => format!("Ubuntu {}", v.as_ref().map(|v| v.to_string()).unwrap_or_default()),
                Dist::Debian(v) => format!("Debian {}", v.as_ref().map(|v| v.to_string()).unwrap_or_default()),
                Dist::Fedora(v) => format!("Fedora {}", v.as_ref().map(|v| v.to_string()).unwrap_or_default()),
                Dist::Tumbleweed => "openSUSE Tumbleweed".to_string(),
                Dist::Leap(v) => format!("openSUSE Leap {}", v.as_ref().map(|v| v.to_string()).unwrap_or_default()),
            });

            let pkg_event = PackageEvent {
                file_name: asset.name.clone(),
                package_type: package_type.to_string(),
                distro: distro_str,
                architecture: package.architecture().to_string(),
                name: package.name().map(|s| s.to_string()),
            };

            let _ = tx
                .send(
                    Event::default()
                        .event("package")
                        .json_data(&pkg_event)
                        .unwrap(),
                )
                .await;

            // Track detected platforms
            match (package.ty(), package.distribution()) {
                (Type::Deb, Some(Dist::Ubuntu(_))) => {
                    detected_platforms.insert("ubuntu".to_string());
                }
                (Type::Deb, Some(Dist::Debian(_))) => {
                    detected_platforms.insert("debian".to_string());
                }
                (Type::Deb, None) => {
                    detected_platforms.insert("ubuntu".to_string());
                    detected_platforms.insert("debian".to_string());
                }
                (Type::Rpm, Some(Dist::Fedora(_))) => {
                    detected_platforms.insert("fedora".to_string());
                }
                (Type::Rpm, Some(Dist::Tumbleweed | Dist::Leap(_))) => {
                    detected_platforms.insert("opensuse".to_string());
                }
                (Type::Rpm, None) => {
                    detected_platforms.insert("fedora".to_string());
                    detected_platforms.insert("opensuse".to_string());
                }
                _ => {}
            }
        }

        // Send link events for detected platforms
        let links: Vec<(&str, &str, String)> = vec![
            ("ubuntu", "Ubuntu Derivatives", format!("https://packhub.dev/sh/ubuntu/github/{owner}/{repo}")),
            ("debian", "Debian Derivatives", format!("https://packhub.dev/sh/debian/github/{owner}/{repo}")),
            ("fedora", "Fedora", format!("https://packhub.dev/sh/yum/github/{owner}/{repo}")),
            ("opensuse", "openSUSE", format!("https://packhub.dev/sh/zypp/github/{owner}/{repo}")),
        ];

        for (platform, label, url) in links {
            if detected_platforms.contains(platform) {
                let link_event = LinkEvent {
                    platform: platform.to_string(),
                    label: label.to_string(),
                    url,
                };

                let _ = tx
                    .send(
                        Event::default()
                            .event("link")
                            .json_data(&link_event)
                            .unwrap(),
                    )
                    .await;
            }
        }

        let _ = tx
            .send(
                Event::default()
                    .event("done")
                    .json_data(serde_json::json!({ "status": "complete" }))
                    .unwrap(),
            )
            .await;
    });

    let stream = unfold(rx, |mut rx| async move {
        let event = rx.recv().await?;
        Some((Ok(event), rx))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub fn sse_routes() -> Router<AppState> {
    Router::new().route("/detect", get(sse_detect_handler))
}
