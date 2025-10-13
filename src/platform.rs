use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use bson::doc;
use chrono::{DateTime, Utc};
use lenient_semver::parse;
use mongodb::Collection;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::{REQWEST, utils::Dist};

static PRE_RELEASE_STRIPER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)\D").unwrap());
static APT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Debian APT.+\((.+)\)"#).unwrap());
static FEDORA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"libdnf \(Fedora Linux (\d+);"#).unwrap());
static TUMBLEWEED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"ZYpp.+"#).unwrap());

/// Detects platform based on the user-agent string of `apt` package manager.
/// 
/// This struct fetches data from [repology](https://repology.org) and stores it in a MongoDB collection.
/// It provides methods to detect the distribution and version based on the `apt` version found in
/// the user-agent string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AptPlatformDetection {
    ubuntu: HashMap<VersionReq, Dist>,
    debian: HashMap<VersionReq, Dist>,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    created_at: DateTime<Utc>,
}

impl AptPlatformDetection {
    /// Fetches the data from repology and updates the MongoDB collection.
    pub async fn update(db: &Collection<AptPlatformDetection>) {
        let data = REQWEST
            .get("https://repology.org/api/v1/project/apt")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        let data: serde_json::Value = serde_json::from_str(&data).unwrap();

        // HashSet to remove duplicates
        let mut map: HashMap<&str, HashSet<Version>> = HashMap::new();

        for item in data.as_array().unwrap() {
            let repo = item["repo"].as_str().unwrap();
            if repo.starts_with("ubuntu") || repo.starts_with("debian") {
                let repo = repo.trim_end_matches("_proposed");
                let ver = item["version"].as_str().unwrap();
                let parsed = fresh_version(parse(ver).unwrap());
                map.entry(repo).or_default().insert(parsed);
            }
        }

        let mut ubuntu = HashMap::new();
        let mut debian = HashMap::new();

        for (key, value) in map.into_iter() {
            let mut versions = value.into_iter().collect::<Vec<Version>>();
            versions.sort();
            let requirement;

            if versions.len() > 1 {
                requirement = VersionReq::parse(&format!(
                    ">={}, <={}",
                    versions[0],
                    versions[versions.len() - 1]
                ))
                .unwrap();
            } else {
                requirement = VersionReq::parse(&format!("={}", versions[0])).unwrap();
            }

            if key.starts_with("ubuntu") {
                let ver = key.trim_start_matches("ubuntu_");
                ubuntu.insert(requirement, Dist::ubuntu(&ver.replace("_", ".")));
            } else if key.starts_with("debian") {
                let ver = key.trim_start_matches("debian_");
                debian.insert(requirement, Dist::debian(ver));
            }
        }

        let data = Self { ubuntu, debian, created_at: Utc::now() };
        
        db.insert_one(&data).await.unwrap();
    }
    
    /// Retrieves the latest data from the MongoDB collection.
    pub async fn retrieve(db: &Collection<AptPlatformDetection>) -> Option<Self> {
        db.find_one(doc!{}).sort(doc! { "created_at": -1 }).await.unwrap()
    }

    pub fn detect_ubuntu_for_apt(&self, agent: &str) -> Dist {
        let ver = get_apt_version(agent);
        let mut dist = Dist::Ubuntu(None);

        let apt = fresh_version(parse(ver).unwrap());

        for (matcher, dst) in self.ubuntu.iter() {
            if matcher.matches(&apt) {
                dist = dst.clone();
                break;
            }
        }

        dist
    }

    pub fn detect_debian_for_apt(&self, agent: &str) -> Dist {
        let ver = get_apt_version(agent);
        let mut dist = Dist::Debian(None);

        let apt = fresh_version(parse(ver).unwrap());

        for (matcher, dst) in self.debian.iter() {
            if matcher.matches(&apt) {
                dist = dst.clone();
                break;
            }
        }

        dist
    }
}

/// Removes the errorneous pre-release or build part from the version.
fn fresh_version(mut ver: Version) -> Version {
    ver.build = semver::BuildMetadata::EMPTY;

    let pre = ver.pre.as_str();

    // `1.0.1ubuntu2.24` is erroneously parsed as `1.0.0-1ubuntu2.24`
    // so we need to strip the pre-release part and set the patch correctly
    if let Some(capture) = PRE_RELEASE_STRIPER.captures(pre) {
        let patch: u64 = capture.get(1).unwrap().as_str().parse().unwrap();
        ver.patch = patch;
        ver.pre = semver::Prerelease::EMPTY;
    }

    ver
}

fn get_apt_version(agent: &str) -> &str {
    APT.captures(agent).unwrap().get(1).unwrap().as_str()
}

/// Retrieve the fedora version from the user-agent string.
pub fn get_fedora_version(agent: &str) -> Option<&str> {
    Some(FEDORA.captures(agent)?.get(1)?.as_str())
}

/// Detect the opensuse fa from the user-agent string.
pub fn detect_opensuse_tumbleweed(agent: &str) -> bool {
    TUMBLEWEED.is_match(agent)
}

pub fn detect_rpm_os(agent: &str) -> Option<Dist> {
    if let Some(ver) = get_fedora_version(agent) {
        Some(Dist::fedora(ver))
    } else if detect_opensuse_tumbleweed(agent) {
        Some(Dist::Tumbleweed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use mongodb::Client;
    use testcontainers_modules::{
        mongo::Mongo,
        testcontainers::{ContainerAsync, runners::AsyncRunner},
    };

    pub async fn setup_mongodb(container: &ContainerAsync<Mongo>) -> Client {
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(27017).await.unwrap();

        mongodb::Client::with_uri_str(&format!("mongodb://{}:{}", host, port))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_match_platform() {
        let container = Mongo::default().start().await.unwrap();
        let client = setup_mongodb(&container).await;
        let db = client.database("repology");
        let collection = db.collection::<AptPlatformDetection>("apt");
        
        AptPlatformDetection::update(&collection).await;
        
        let platform = AptPlatformDetection::retrieve(&collection).await.unwrap();

        // Ubuntu
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.0.2)"),
            Dist::ubuntu("20.04")
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.0.9)"),
            Dist::ubuntu("20.04")
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.4.5)"),
            Dist::ubuntu("22.04")
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.4.8)"),
            Dist::ubuntu("22.04")
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.4.10)"),
            Dist::ubuntu("22.04")
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.7.14build2)"),
            Dist::ubuntu("24.04")
        );

        // Debian
        // assert_eq!(
        //     platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (1.8.2.3)"),
        //     Dist::debian("10")
        // );
        assert_eq!(
            platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (2.2.4)"),
            Dist::debian("11")
        );
        assert_eq!(
            platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (2.6.1)"),
            Dist::debian("12")
        );
        // assert_eq!(
        //     platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (2.9.23)"),
        //     Dist::Debian(Some("13".to_owned()))
        // );
    }

    #[test]
    fn test_apt_version() {
        assert_eq!(get_apt_version("Debian APT-HTTP/1.3 (2.5.3)"), "2.5.3");
    }

    #[test]
    fn test_fedora_version() {
        assert_eq!(
            get_fedora_version("libdnf (Fedora Linux 38; container; Linux.x86_64)"),
            Some("38")
        );
        assert_eq!(
            get_fedora_version("libdnf (Fedora Linux 39; container; Linux.x86_64)"),
            Some("39")
        );
    }

    #[test]
    fn test_detect_opensuse() {
        assert!(detect_opensuse_tumbleweed(
            "ZYpp 17.31.15 (curl 8.5.0) openSUSE-Tumbleweed-x86_64"
        ));
        assert!(detect_opensuse_tumbleweed("ZYpp 17.37.17 (curl 8.15.0)"));
    }
}
