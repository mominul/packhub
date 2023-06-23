use std::sync::Arc;

use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use once_cell::sync::Lazy;

use crate::detect::Package;

static OCTOCRAB: Lazy<Arc<Octocrab>> = Lazy::new(|| octocrab::instance());

struct Repository {
    // project: String,
    // updated: DateTime<Utc>,
    packages: Vec<Package>,
}

impl Repository {
    pub async fn from_github(owner: String, repo: String) -> Self {
        let mut packages = Vec::new();
        let assets = OCTOCRAB
            .repos(owner, repo)
            .releases()
            .get_latest()
            .await
            .unwrap()
            .assets;

        for asset in assets {
            let package = Package::detect_package(&asset.name, asset.url.to_string()).unwrap();
            packages.push(package);
        }

        Repository { packages }
    }
}
