use askama::Template;
use axum::{extract::Path, routing::get, Router};

use crate::state::AppState;

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

async fn apt_script_handler(Path((distro, owner, repo)): Path<(String, String, String)>) -> String {
    generate_apt_script(&distro, &owner, &repo)
}

pub fn script_routes() -> Router<AppState> {
    Router::new().route("/{distro}/github/{owner}/{repo}", get(apt_script_handler))
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
}
