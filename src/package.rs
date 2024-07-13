use std::sync::{Arc, Mutex};

use anyhow::{bail, Result};
use chrono::{DateTime, Utc};

use crate::utils::{Dist, Type};

struct InnerPackage {
    tipe: Type,
    dist: Option<Dist>,
    url: String,
    ver: String,
    data: Mutex<Data<Vec<u8>>>,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
}

#[derive(Clone, PartialEq)]
pub struct Package {
    inner: Arc<InnerPackage>,
}

/// Data type for package data and metadata.
///
/// It is used to differentiate between package data and metadata.
///
/// `Data::Package` is used for package data. It is the actual package
/// file which needs to be processed to extract the metadata.
///
/// `Data::Metadata` is used for package metadata.
///
/// `Data::None` is used when no data is available.
#[derive(Clone, PartialEq)]
pub enum Data<T> {
    Package(T),
    Metadata(T),
    None,
}

impl std::fmt::Debug for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name())
    }
}

impl PartialEq for InnerPackage {
    fn eq(&self, other: &Self) -> bool {
        self.tipe == other.tipe
            && self.dist == other.dist
            && self.url == other.url
            && self.ver == other.ver
            && *self.data.lock().unwrap() == *other.data.lock().unwrap()
            && self.created == other.created
            && self.updated == other.updated
    }
}

impl Package {
    pub fn detect_package(
        name: &str,
        ver: String,
        url: String,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
    ) -> Result<Package> {
        // Split the extension first.
        // If we don't recognize it, then return error.
        let Some((tipe, splitted)) = split_extention(name) else {
            bail!("Unknown package type: {}", name);
        };

        let mut dist: Option<Dist> = None;
        let sections: Vec<&str> = splitted.split(['-', '_']).collect();

        for section in sections {
            match section {
                dst if dst.contains("ubuntu") => dist = Some(Dist::Ubuntu(parse_version(dst))),
                dst if dst.contains("debian") => dist = Some(Dist::Debian(parse_version(dst))),
                dst if dst.contains("fedora") => dist = Some(Dist::Fedora(parse_version(dst))),
                _ => (),
            }
        }

        let inner = InnerPackage {
            tipe,
            dist,
            url,
            ver,
            data: Mutex::new(Data::None),
            created,
            updated,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn ty(&self) -> &Type {
        &self.inner.tipe
    }

    /// Return the distribution for which it was packaged
    pub fn distribution(&self) -> &Option<Dist> {
        &self.inner.dist
    }

    /// Version of the package
    pub fn version(&self) -> &str {
        &self.inner.ver
    }

    pub fn download_url(&self) -> &str {
        &self.inner.url
    }

    pub fn file_name(&self) -> &str {
        &self.inner.url.split('/').last().unwrap()
    }

    /// Download package data
    ///
    /// It is required to call this function before calling the `data()` function.
    pub async fn download(&self) -> Result<()> {
        let data = reqwest::get(self.download_url()).await?.bytes().await?;
        *self.inner.data.lock().unwrap() = Data::Package(data.to_vec());
        Ok(())
    }

    /// Return the data of the package.
    ///
    /// It is required to call the `download()` or `set_metadata()` function before calling this.
    /// Otherwise, `None` is returned.
    pub fn data(&self) -> Data<Vec<u8>> {
        self.inner.data.lock().unwrap().clone()
    }

    #[cfg(test)]
    /// Set the package data.
    ///
    /// It's for testing purpose.
    pub fn set_package_data(&self, data: Vec<u8>) {
        *self.inner.data.lock().unwrap() = Data::Package(data);
    }

    pub fn creation_date(&self) -> &DateTime<Utc> {
        &self.inner.created
    }

    pub fn updated_date(&self) -> &DateTime<Utc> {
        &self.inner.updated
    }

    /// Set the package metadata.
    pub fn set_metadata(&self, metadata: Vec<u8>) {
        *self.inner.data.lock().unwrap() = Data::Metadata(metadata);
    }

    /// Check if metadata is available.
    pub fn is_metadata_available(&self) -> bool {
        matches!(*self.inner.data.lock().unwrap(), Data::Metadata(_))
    }
}

/// Parses the version from the distribution identifier `dist`.
///
/// For instance, for a distribution identifier `ubuntu22.10` it will
/// parse the version as `22.10`.
fn parse_version(dist: &str) -> Option<String> {
    split_at_numeric(dist).map(|s| s.to_owned())
}

/// Splits the string `s` at the first occurence of a numeric digit.
///
/// It is used to extract version number from strings, such as for "ubuntu24.10" it would
/// return "24.10".
fn split_at_numeric(s: &str) -> Option<&str> {
    for (curr, (index, next)) in s.chars().zip(s.char_indices().skip(1)) {
        if curr.is_ascii_alphabetic() && next.is_ascii_digit() {
            return Some(&s[index..]);
        }
    }

    None
}

fn split_extention(s: &str) -> Option<(Type, &str)> {
    let mut str = String::with_capacity(3);
    let mut index = 0;

    for (idx, ch) in s.char_indices().rev() {
        if ch == '.' {
            index = idx;
            break;
        } else {
            str.push(ch);
        }
    }

    if index == 0 {
        return None;
    }

    let splitted = &s[0..index];

    // `str` is in reverse order, so we try to match it reversely.
    let tipe = match str.as_str() {
        "bed" => Type::Deb,
        "mpr" => Type::Rpm,
        _ => return None,
    };

    Some((tipe, splitted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package() {
        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "2.0.0");
        assert_eq!(
            *pack.distribution(),
            Some(Dist::Ubuntu(Some("22.04".to_owned())))
        );
        assert_eq!(*pack.ty(), Type::Deb);

        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-fedora36.rpm",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "2.0.0");
        assert_eq!(
            *pack.distribution(),
            Some(Dist::Fedora(Some("36".to_owned())))
        );
        assert_eq!(*pack.ty(), Type::Rpm);

        let pack = Package::detect_package(
            "caprine_2.56.1_amd64.deb",
            "v2.56.1".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "v2.56.1");
        assert_eq!(*pack.distribution(), None);
        assert_eq!(*pack.ty(), Type::Deb);
    }

    #[test]
    fn test_package_change_propagation() {
        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
            DateTime::UNIX_EPOCH,
        )
        .unwrap();

        assert!(!pack.is_metadata_available());

        let pack2 = pack.clone();
        pack2.set_metadata(Vec::new());

        assert!(pack.is_metadata_available());
    }

    #[test]
    fn test_split_extension() {
        assert_eq!(
            split_extention("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb"),
            Some((Type::Deb, "OpenBangla-Keyboard_2.0.0-ubuntu22.04"))
        );
        assert_eq!(
            split_extention("OpenBangla-Keyboard_2.0.0-fedora36.rpm"),
            Some((Type::Rpm, "OpenBangla-Keyboard_2.0.0-fedora36"))
        );
        assert_eq!(split_extention("caprine_2.56.1_amd64.snap"), None);
        assert_eq!(split_extention("deb"), None);
    }

    #[test]
    fn test_split_test() {
        assert_eq!(split_at_numeric("ubuntu24.10"), Some("24.10"));
        assert_eq!(split_at_numeric("ubuntu"), None);
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("ubuntu22.10").unwrap(), "22.10".to_owned());
        assert_eq!(parse_version("fedora37").unwrap(), "37".to_owned());
    }
}
