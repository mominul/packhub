use std::{ops::Add, str::FromStr};

use anyhow::Result;
use sha1::digest::{generic_array::ArrayLength, Digest, OutputSizeUser};

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
