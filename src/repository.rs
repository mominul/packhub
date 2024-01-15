use std::sync::Arc;

use octocrab::Octocrab;
use once_cell::sync::Lazy;

use crate::{package::Package, platform::get_apt_version, selector::select_package_ubuntu};

static OCTOCRAB: Lazy<Arc<Octocrab>> = Lazy::new(|| octocrab::instance());

pub struct Repository {
    // project: String,
    // updated: DateTime<Utc>,
    packages: Vec<Package>,
}

impl Repository {
    pub async fn from_github(owner: String, repo: String) -> Self {
        let mut packages = Vec::new();
        let release = OCTOCRAB
            .repos(owner, repo)
            .releases()
            .get_latest()
            .await
            .unwrap();

        for asset in release.assets {
            let package = Package::detect_package(
                &asset.name,
                release.tag_name.clone(),
                asset.browser_download_url.to_string(),
                asset.updated_at,
            );
            if let Ok(package) = package {
                packages.push(package);
            }
        }

        Repository { packages }
    }

    pub fn select_package_ubuntu(&self, agent: &str) -> &Package {
        let apt = get_apt_version(agent);
        select_package_ubuntu(&self.packages, apt)
    }
}
