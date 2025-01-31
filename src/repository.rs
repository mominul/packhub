use std::sync::{Arc, LazyLock};

use anyhow::{bail, Result};
use mongodb::{Client, Collection};
use octocrab::Octocrab;
use tokio::task::JoinSet;
use tracing::{debug, error};

use crate::{
    db::PackageMetadata,
    package::Package,
    platform::{detect_rpm_os, AptPlatformDetection},
    selector::select_packages,
};

static OCTOCRAB: LazyLock<Arc<Octocrab>> = LazyLock::new(|| octocrab::instance());

pub struct Repository {
    collection: Collection<PackageMetadata>,
    packages: Vec<Package>,
    downloaded: Vec<Package>,
    platform: AptPlatformDetection,
}

impl Repository {
    pub async fn from_github(owner: String, repo: String, client: Client) -> Self {
        let project = format!("{owner}/{repo}");
        let collection = client
            .database("github")
            .collection::<PackageMetadata>(&project);

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
                if let Some(metadata) = PackageMetadata::retrieve_from(&collection, &package).await
                {
                    package.set_metadata(metadata.data());
                }
                packages.push(package);
            }
        }

        let platform = AptPlatformDetection::initialize().await;

        Repository {
            collection,
            packages,
            platform,
            downloaded: Vec::new(),
        }
    }

    pub async fn save_package_metadata(&mut self) {
        for package in &self.downloaded {
            let Some(metadata) = PackageMetadata::from_package(package) else {
                error!(
                    "Metadata was not available for saving the package: {:?}",
                    package.file_name()
                );
                return;
            };

            if let Err(e) = self.collection.insert_one(metadata).await {
                error!(
                    "Failed to save metadata for package: {:?}\n Error: {e}",
                    package.file_name()
                );
                return;
            };
            debug!("Saved metadata for package: {:?}", package.file_name());
        }
    }

    /// Select packages for apt based distributions.
    ///
    /// The `distro` parameter is the name of the distribution (`debian`, `ubuntu`).
    ///
    /// The `agent` parameter is the user-agent string of the apt client.
    ///
    /// It returns a vector of packages that are compatible with the given agent.
    ///
    /// It also downloads the selected packages if the metadata is not available.
    pub async fn select_package_apt(&mut self, distro: &str, agent: &str) -> Result<Vec<Package>> {
        let dist = match distro {
            "ubuntu" => self.platform.detect_ubuntu_for_apt(agent),
            "debian" => self.platform.detect_debian_for_apt(agent),
            dist => bail!("Unknown apt distribution {dist}"),
        };

        let packages: Vec<Package> = select_packages(&self.packages, dist)
            .into_iter()
            .map(|p| p.clone())
            .collect();

        debug!("Packages selected {:?}", packages);

        self.download_packages(packages).await
    }

    /// Select packages for RPM based distribution.
    ///
    /// The `agent` parameter is the user-agent string of the rpm client.
    ///
    /// It returns a vector of packages that are compatible with the given agent.
    ///
    /// It also downloads the selected packages if the metadata is not available.
    pub async fn select_package_rpm(&mut self, agent: &str) -> Result<Vec<Package>> {
        let Some(dist) = detect_rpm_os(agent) else {
            bail!("Unknown RPM distribution agent: {agent}");
        };
        let packages: Vec<Package> = select_packages(&self.packages, dist)
            .into_iter()
            .map(|p| p.clone())
            .collect();

        debug!("Packages selected {:?}", packages);

        self.download_packages(packages).await
    }

    async fn download_packages(&mut self, packages: Vec<Package>) -> Result<Vec<Package>> {
        let mut runner = JoinSet::new();
        let mut result = Vec::new();

        for package in packages {
            if !package.is_metadata_available() {
                runner.spawn(async move {
                    debug!("Downloading package: {:?}", package.file_name());
                    package.download().await.and_then(|_| Ok(package))
                });
            } else {
                debug!("Package metadata available: {:?}", package.file_name());
                result.push(package);
            }
        }

        while let Some(res) = runner.join_next().await {
            let Ok(res) = res else {
                bail!("Executor error: Failed to download package")
            };

            let package = res?;

            debug!("Downloaded package: {:?}", package.file_name());

            result.push(package.clone());
            self.downloaded.push(package);
        }

        result.sort();

        Ok(result)
    }
}
