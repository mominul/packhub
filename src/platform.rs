use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use lenient_semver::parse;
use regex::Regex;
use semver::{Version, VersionReq};

use crate::{utils::Dist, REQWEST};

static PRE_RELEASE_STRIPER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)\D").unwrap());
static APT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Debian APT.+\((.+)\)"#).unwrap());
static FEDORA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"libdnf \(Fedora Linux (\d+);"#).unwrap());
static TUMBLEWEED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"ZYpp.+openSUSE-Tumbleweed"#).unwrap());

/// Detects platform based on the user-agent string of `apt` package manager.
pub struct AptPlatformDetection {
    ubuntu: HashMap<VersionReq, Dist>,
    debian: HashMap<VersionReq, Dist>,
}

impl AptPlatformDetection {
    pub async fn initialize() -> Self {
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
                ubuntu.insert(requirement, Dist::Ubuntu(Some(ver.replace("_", "."))));
            } else if key.starts_with("debian") {
                let ver = key.trim_start_matches("debian_");
                debian.insert(requirement, Dist::Debian(Some(ver.to_owned())));
            }
        }

        Self { ubuntu, debian }
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

fn get_apt_version<'a>(agent: &'a str) -> &'a str {
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
        Some(Dist::Fedora(Some(ver.to_owned())))
    } else if detect_opensuse_tumbleweed(agent) {
        Some(Dist::Tumbleweed)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_match_platform() {
        let platform = AptPlatformDetection::initialize().await;

        // Ubuntu
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.0.2)"),
            Dist::Ubuntu(Some("20.04".to_owned()))
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.0.9)"),
            Dist::Ubuntu(Some("20.04".to_owned()))
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.4.5)"),
            Dist::Ubuntu(Some("22.04".to_owned()))
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.4.8)"),
            Dist::Ubuntu(Some("22.04".to_owned()))
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.4.10)"),
            Dist::Ubuntu(Some("22.04".to_owned()))
        );
        assert_eq!(
            platform.detect_ubuntu_for_apt("Debian APT-HTTP/1.3 (2.7.14build2)"),
            Dist::Ubuntu(Some("24.04".to_owned()))
        );

        // Debian
        assert_eq!(
            platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (1.8.2.3)"),
            Dist::Debian(Some("10".to_owned()))
        );
        assert_eq!(
            platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (2.2.4)"),
            Dist::Debian(Some("11".to_owned()))
        );
        assert_eq!(
            platform.detect_debian_for_apt("Debian APT-HTTP/1.3 (2.6.1)"),
            Dist::Debian(Some("12".to_owned()))
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
    }
}
