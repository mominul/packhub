use anyhow::anyhow;
use askama::Template;
use axum::{extract::Path, routing::get, Router};

use crate::{error::AppError, state::AppState};

#[derive(Template)]
#[template(path = "apt-script.sh", escape = "none")]
struct AptScript<'a> {
    host: &'a str,
    distro: &'a str,
    owner: &'a str,
    repo: &'a str,
}

fn generate_apt_script(distro: &str, owner: &str, repo: &str) -> String {
    let host = dotenvy::var("PACKHUB_DOMAIN").unwrap();
    let script = AptScript {
        host: &host,
        distro,
        owner,
        repo,
    };
    script.render().unwrap()
}

#[derive(Template)]
#[template(path = "rpm-script.sh", escape = "none")]
struct RPMScript<'a> {
    host: &'a str,
    owner: &'a str,
    repo: &'a str,
}

fn generate_rpm_script(owner: &str, repo: &str) -> String {
    let host = dotenvy::var("PACKHUB_DOMAIN").unwrap();
    let script = RPMScript {
        host: &host,
        owner,
        repo,
    };
    script.render().unwrap()
}

async fn script_handler(
    Path((distro, owner, repo)): Path<(String, String, String)>,
) -> Result<String, AppError> {
    match distro.as_str() {
        "ubuntu" | "debian" => Ok(generate_apt_script(&distro, &owner, &repo)),
        "rpm" => Ok(generate_rpm_script(&owner, &repo)),
        _ => Err(anyhow!("Script Generation: Unsupported distro: {}", distro).into()),
    }
}

pub fn script_routes() -> Router<AppState> {
    Router::new().route("/{distro}/github/{owner}/{repo}", get(script_handler))
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_script_generation_apt() {
        let apt_script = generate_apt_script("ubuntu", "OpenBangla", "OpenBangla-Keyboard");
        assert_snapshot!(apt_script);
    }

    #[test]
    fn test_script_generation_rpm() {
        let script = generate_rpm_script("OpenBangla", "OpenBangla-Keyboard");
        assert_snapshot!(script);
    }
}
