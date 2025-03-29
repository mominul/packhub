use std::{ops::Add, str::FromStr};

use anyhow::Result;
use lenient_semver::parse;
use semver::Version;
use sha1::digest::{generic_array::ArrayLength, Digest, OutputSizeUser};

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
            Dist::Debian(ref mut ver) => *ver = version.and_then(|v| parse(v).ok()),
            Dist::Ubuntu(ref mut ver) => *ver = version.and_then(|v| parse(v).ok()),
            Dist::Fedora(ref mut ver) => *ver = version.and_then(|v| parse(v).ok()),
            Dist::Tumbleweed => {}
            Dist::Leap(ref mut ver) => *ver = version.and_then(|v| parse(v).ok()),
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

#[derive(Debug, PartialEq, Eq, Clone, Hash, Default)]
pub enum Arch {
    #[default]
    Amd64,
    Arm64,
    Armhf,
    Armv7,
    Aarch64,
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
            "armv7" => Ok(Arch::Armv7),
            _ => Err(()),
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
