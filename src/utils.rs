use std::{ops::Add, str::FromStr};

use anyhow::{bail, Result};
use sha1::digest::{generic_array::ArrayLength, Digest, OutputSizeUser};
use tokio::task::JoinSet;
use tracing::debug;

use crate::package::Package;

#[derive(Debug, PartialEq, Clone)]
pub enum Dist {
    Ubuntu(Option<String>),
    Debian(Option<String>),
    Fedora(Option<String>),
}

#[derive(Debug, PartialEq)]
pub enum Arch {
    Amd64,
}

impl FromStr for Arch {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "amd64" => Ok(Arch::Amd64),
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
            Type::Rpm => matches!(dist, Dist::Fedora(_)),
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

/// Parallelly download packages
pub async fn download_packages(packages: Vec<Package>) -> Result<Vec<Package>> {
    let mut runner = JoinSet::new();

    for package in packages {
        runner.spawn(async move {
            debug!("Downloading package: {:?}", package.file_name());
            package.download().await.and_then(|_| Ok(package))
        });
    }

    let mut result = Vec::new();

    while let Some(res) = runner.join_next().await {
        let Ok(res) = res else {
            bail!("Executor error: Failed to download package")
        };

        let package = res?;

        debug!("Downloaded package: {:?}", package.file_name());

        result.push(package);
    }

    Ok(result)
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
    }
}
