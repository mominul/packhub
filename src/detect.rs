/// This module is responsible for inferring the distribution and version from a given filename.
use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;

use crate::utils::{Arch, Dist};

// Regex to capture the package name (stops before version numbers)
static NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^([a-z0-9_.-]+?)(?:[-_](?:v?\d.*))").unwrap());

// Regex to capture architecture
static ARCH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(x86_64|amd64|aarch64|arm64|armhf|armv7)").unwrap());

static DISTRO_PATTERNS: LazyLock<Vec<(Regex, Dist)>> = LazyLock::new(|| {
    vec![
        // Fedora (fc followed by digits)
        (Regex::new(r"fc(\d+)").unwrap(), Dist::Fedora(None)),
        // Fedora (fedora followed by optional hyphen and digits)
        (Regex::new(r"fedora-?(\d+)?").unwrap(), Dist::Fedora(None)),
        // openSUSE Leap (lp followed by digits and decimal)
        (Regex::new(r"lp(\d+\.\d+)").unwrap(), Dist::Leap(None)),
        // openSUSE Leap (opensuse-leap followed by optional hyphen and version)
        (
            Regex::new(r"opensuse-leap-?(\d+\.\d+)").unwrap(),
            Dist::Leap(None),
        ),
        // Debian (debian followed by optional hyphen and digits)
        (Regex::new(r"debian-?(\d+)").unwrap(), Dist::Debian(None)),
        // Debian (without version)
        (Regex::new(r"debian").unwrap(), Dist::Debian(None)),
        // Ubuntu (ubuntu followed by optional hyphen and digits with decimal)
        (
            Regex::new(r"ubuntu-?(\d+\.\d+)").unwrap(),
            Dist::Ubuntu(None),
        ),
        // Ubuntu (ubuntu followed by optional hyphen and codename)
        (Regex::new(r"ubuntu-?([a-z]+)").unwrap(), Dist::Ubuntu(None)),
        // Ubuntu (without version)
        (Regex::new(r"ubuntu").unwrap(), Dist::Ubuntu(None)),
        (Regex::new(r"(?i)suse").unwrap(), Dist::Tumbleweed),
        (Regex::new(r"(tw|tumbleweed)").unwrap(), Dist::Tumbleweed),
    ]
});

static UBUNTU_CODENAMES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    [
        ("precise", "12.04"),
        ("trusty", "14.04"),
        ("xenial", "16.04"),
        ("bionic", "18.04"),
        ("focal", "20.04"),
        ("jammy", "22.04"),
        ("lunar", "23.04"),
        ("mantic", "23.10"),
        ("noble", "24.04"),
    ]
    .into()
});

/// Package information gained from the filename
#[derive(Debug, PartialEq)]
pub struct PackageInfo {
    pub name: Option<String>,
    pub distro: Option<Dist>,
    pub architecture: Option<Arch>,
}

impl PackageInfo {
    pub fn parse_package(filename: &str) -> PackageInfo {
        let mut name = None;
        let mut architecture = None;
        let mut distro = None;

        // Extract package name (e.g., "notes" from "notes-2.3.1...")
        if let Some(caps) = NAME_RE.captures(filename) {
            name = caps.get(1).map(|m| m.as_str().to_string());
        }

        // As we have the name, we can remove it from the filename.
        // To reduce the chance of other regexes matching the name.
        let filename = filename.trim_start_matches(name.as_deref().unwrap_or_default());

        // Extract architecture (e.g., "x86_64")
        if let Some(caps) = ARCH_RE.captures(filename) {
            architecture = caps
                .get(1)
                .and_then(|m| m.as_str().to_lowercase().parse().ok());
        }

        // Extract distro and version (e.g., "fedora" and "38")
        for (re, dist) in DISTRO_PATTERNS.iter() {
            if let Some(caps) = re.captures(filename) {
                let mut dist = dist.clone();
                let version = caps.get(1).map(|m| m.as_str());

                // Check if the distro is Ubuntu and map the codename to version
                if let Dist::Ubuntu(_) = dist {
                    if let Some(codename) = version {
                        if let Some(ver) = UBUNTU_CODENAMES.get(codename) {
                            dist.set_version(Some(ver));
                        } else {
                            dist.set_version(Some(codename));
                        }
                    }
                } else {
                    dist.set_version(version);
                }

                distro = Some(dist);
                break; // Stop at first match
            }
        }

        PackageInfo {
            name,
            distro,
            architecture,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_extraction() {
        let cases = vec![
            ("notes-2.3.1-1.x86_64.rpm", "notes"),
            ("OpenBangla-Keyboard_2.0.0.deb", "OpenBangla-Keyboard"),
            ("caprine_2.60.3_amd64.deb", "caprine"),
            ("rustdesk-1.3.8-aarch64.deb", "rustdesk"),
            ("myapp-v1.2.3.deb", "myapp"),
            ("special_pkg-v2.3_arm64.deb", "special_pkg"),
        ];

        for (filename, expected) in cases {
            let info = PackageInfo::parse_package(filename);
            assert_eq!(
                info.name,
                Some(expected.to_string()),
                "Failed for: {}",
                filename
            );
        }
    }

    #[test]
    fn test_fedora_38() {
        let info = PackageInfo::parse_package("notes-2.3.1-1.x86_64-qt6-fedora-38.rpm");
        assert_eq!(info.name, Some("notes".into()));
        assert_eq!(info.distro, Some(Dist::fedora("38")));
        assert_eq!(info.architecture, Some(Arch::Amd64));
    }

    #[test]
    fn test_fedora38_short() {
        let info = PackageInfo::parse_package("OpenBangla-Keyboard_2.0.0-fedora38.rpm");
        assert_eq!(info.name, Some("OpenBangla-Keyboard".into()));
        assert_eq!(info.distro, Some(Dist::fedora("38")));
        assert_eq!(info.architecture, None);
    }

    #[test]
    fn test_opensuse_leap() {
        let info = PackageInfo::parse_package("flameshot-12.1.0-1-lp15.2.x86_64.rpm");
        assert_eq!(info.name, Some("flameshot".into()));
        assert_eq!(info.distro, Some(Dist::leap("15.2")));
        assert_eq!(info.architecture, Some(Arch::Amd64));
    }

    #[test]
    fn test_ubuntu_jammy() {
        let info = PackageInfo::parse_package("notes_2.3.1_amd64-qt6-ubuntu-jammy.deb");
        assert_eq!(info.name, Some("notes".into()));
        assert_eq!(info.distro, Some(Dist::ubuntu("22.04")));
        assert_eq!(info.architecture, Some(Arch::Amd64));
    }

    #[test]
    fn test_debian10() {
        let info = PackageInfo::parse_package("flameshot-12.1.0-1.debian-10.amd64.deb");
        assert_eq!(info.name, Some("flameshot".into()));
        assert_eq!(info.distro, Some(Dist::debian("10")));
        assert_eq!(info.architecture, Some(Arch::Amd64));
    }

    #[test]
    fn test_suse() {
        let info = PackageInfo::parse_package("rustdesk-1.3.8-0.aarch64-suse.rpm");
        assert_eq!(info.name, Some("rustdesk".into()));
        assert_eq!(info.distro, Some(Dist::Tumbleweed));
        assert_eq!(info.architecture, Some(Arch::Aarch64));
    }

    #[test]
    fn test_opensuse_tumbleweed_full_pattern() {
        let filename = "another-package-2.3.4-1.tumbleweed.noarch.rpm";
        let result = PackageInfo::parse_package(filename);
        assert_eq!(result.name, Some("another-package".into()));
        assert_eq!(result.distro, Some(Dist::Tumbleweed));
    }

    #[test]
    fn test_distro_without_version() {
        let info = PackageInfo::parse_package("ibus-openbangla_3.0.0-fedora.rpm");
        assert_eq!(info.name, Some("ibus-openbangla".into()));
        assert_eq!(info.distro, Some(Dist::Fedora(None)));
        assert_eq!(info.architecture, None);

        let info = PackageInfo::parse_package("ibus-openbangla_3.0.0-ubuntu.deb");
        assert_eq!(info.distro, Some(Dist::Ubuntu(None)));

        let info = PackageInfo::parse_package("ibus-openbangla_3.0.0-debian.deb");
        assert_eq!(info.distro, Some(Dist::Debian(None)));
    }

    #[test]
    fn test_caprine() {
        let info = PackageInfo::parse_package("caprine_2.60.3_amd64.deb");
        assert_eq!(info.name, Some("caprine".into()));
        assert_eq!(info.distro, None); // No distro pattern matched
        assert_eq!(info.architecture, Some(Arch::Amd64));
    }
}
