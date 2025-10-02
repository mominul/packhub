use std::{fmt::Display, ops::Add, str::FromStr};

use anyhow::Result;
use lenient_semver::parse;
use semver::Version;
use serde::Deserialize;
use sha1::digest::{Digest, OutputSizeUser, generic_array::ArrayLength};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Dist {
    Ubuntu(Option<Version>),
    Debian(Option<Version>),
    Fedora(Option<Version>),
    Tumbleweed,
    Leap(Option<Version>),
}

impl Dist {
    /// Check if it matches the `dist` without regarding its version.
    ///
    /// A loose check than `==`.
    pub fn matches_distribution(&self, dist: &Dist) -> bool {
        match self {
            Dist::Debian(_) => matches!(dist, Dist::Debian(_)),
            Dist::Ubuntu(_) => matches!(dist, Dist::Ubuntu(_)),
            Dist::Fedora(_) => matches!(dist, Dist::Fedora(_)),
            Dist::Tumbleweed => matches!(dist, Dist::Tumbleweed),
            Dist::Leap(_) => matches!(dist, Dist::Leap(_)),
        }
    }

    pub fn set_version(&mut self, version: Option<&str>) {
        match self {
            Dist::Debian(ver) => *ver = version.and_then(|v| parse(v).ok()),
            Dist::Ubuntu(ver) => *ver = version.and_then(|v| parse(v).ok()),
            Dist::Fedora(ver) => *ver = version.and_then(|v| parse(v).ok()),
            Dist::Tumbleweed => {}
            Dist::Leap(ver) => *ver = version.and_then(|v| parse(v).ok()),
        }
    }

    pub fn ubuntu(version: &str) -> Self {
        Dist::Ubuntu(parse(version).ok())
    }

    pub fn debian(version: &str) -> Self {
        Dist::Debian(parse(version).ok())
    }

    pub fn fedora(version: &str) -> Self {
        Dist::Fedora(parse(version).ok())
    }

    pub fn leap(version: &str) -> Self {
        Dist::Leap(parse(version).ok())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub enum Arch {
    #[default]
    Amd64,
    Arm64,
    Armhf, // Hard Float ABI ARM. Compatible with armv7 and armv6
    Armv7,
    Aarch64,
    PPC64le,
    RiscV64,
    S390x,
}

impl FromStr for Arch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "amd64" => Ok(Arch::Amd64),
            "x86_64" => Ok(Arch::Amd64),
            "aarch64" => Ok(Arch::Aarch64),
            "arm64" => Ok(Arch::Arm64),
            "armhf" => Ok(Arch::Armhf),
            "armv6l" => Ok(Arch::Armhf),
            "armv7" => Ok(Arch::Armv7),
            "armv7l" => Ok(Arch::Armv7),
            "ppc64le" => Ok(Arch::PPC64le),
            "riscv64" => Ok(Arch::RiscV64),
            "s390x" => Ok(Arch::S390x),
            _ => Err(()),
        }
    }
}

// Currently follows the naming convention of Debian
// https://www.debian.org/ports/#portlist-released
impl Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::Amd64 => write!(f, "amd64"),
            Arch::Arm64 => write!(f, "arm64"),
            Arch::Armhf => write!(f, "armhf"),
            Arch::Armv7 => write!(f, "armhf"),
            Arch::Aarch64 => write!(f, "arm64"),
            Arch::PPC64le => write!(f, "ppc64el"),
            Arch::RiscV64 => write!(f, "riscv64"),
            Arch::S390x => write!(f, "s390x"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Deb,
    Rpm,
}

impl Type {
    pub fn matches_distribution(&self, dist: &Dist) -> bool {
        match self {
            Type::Deb => matches!(dist, Dist::Debian(_) | Dist::Ubuntu(_)),
            Type::Rpm => matches!(dist, Dist::Fedora(_) | Dist::Tumbleweed),
        }
    }
}

pub fn hashsum<T: Digest>(data: &[u8]) -> String
where
    <T as OutputSizeUser>::OutputSize: Add,
    <<T as OutputSizeUser>::OutputSize as Add>::Output: ArrayLength<u8>,
{
    format!("{:x}", T::digest(data))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Unstable,
}

impl Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseChannel::Stable => write!(f, "stable"),
            ReleaseChannel::Unstable => write!(f, "unstable"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppVersion {
    V1,
    V2,
}

impl Display for AppVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppVersion::V1 => write!(f, "v1"),
            AppVersion::V2 => write!(f, "v2"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_matches_distribution() {
        assert!(Type::Deb.matches_distribution(&Dist::Debian(None)));
        assert!(Type::Deb.matches_distribution(&Dist::Ubuntu(None)));
        assert!(!Type::Deb.matches_distribution(&Dist::Fedora(None)));
        assert!(Type::Rpm.matches_distribution(&Dist::Fedora(None)));
        assert!(Type::Rpm.matches_distribution(&Dist::Tumbleweed));
    }

    #[test]
    fn test_dist_matches() {
        assert!(Dist::Ubuntu(None).matches_distribution(&Dist::ubuntu("24.04")));
        assert!(!Dist::Debian(None).matches_distribution(&Dist::ubuntu("24.04")));
    }

    #[test]
    fn test_dist_version_comparison() {
        let ver1 = Dist::ubuntu("24.04");
        let ver2 = Dist::ubuntu("24.10");
        let ver0 = Dist::Ubuntu(None);

        assert!(ver1 < ver2);
        assert!(ver2 > ver1);
        assert!(ver1 > ver0);
        assert!(ver0 < ver1);

        let ver3 = Dist::fedora("38");
        let ver4 = Dist::fedora("41");
        let ver0 = Dist::Fedora(None);

        assert!(ver3 < ver4);
        assert!(ver4 > ver3);
        assert!(ver3 > ver0);
        assert!(ver0 < ver3);
    }
}
