use std::sync::{Arc, Mutex};

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};

use crate::{
    REQWEST,
    detect::PackageInfo,
    utils::{Arch, Dist, Type},
};

struct InnerPackage {
    tipe: Type,
    info: PackageInfo,
    url: String,
    ver: String,
    data: Mutex<Data>,
    created: DateTime<Utc>,
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
pub enum Data {
    Package(Vec<u8>),
    Metadata(String),
    None,
}

impl std::fmt::Debug for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name())
    }
}

impl Eq for Package {}

impl PartialOrd for Package {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Package {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file_name().cmp(other.file_name())
    }
}

impl PartialEq for InnerPackage {
    fn eq(&self, other: &Self) -> bool {
        self.tipe == other.tipe
            && self.info == other.info
            && self.url == other.url
            && self.ver == other.ver
            && *self.data.lock().unwrap() == *other.data.lock().unwrap()
            && self.created == other.created
    }
}

impl Package {
    pub fn detect_package(
        name: &str,
        ver: String,
        url: String,
        created: DateTime<Utc>,
    ) -> Result<Package> {
        // Split the extension first.
        // If we don't recognize it, then return error.
        let Some(tipe) = split_extention(name) else {
            bail!("Unknown package type: {}", name);
        };

        let info = PackageInfo::parse_package(name);

        let inner = InnerPackage {
            tipe,
            info,
            url,
            ver,
            data: Mutex::new(Data::None),
            created,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Name of the package
    pub fn name(&self) -> Option<&str> {
        self.inner.info.name.as_deref()
    }

    /// Architecture of the package.
    /// By default, it is `amd64`.
    pub fn architecture(&self) -> Arch {
        self.inner
            .info
            .architecture
            .as_ref()
            .cloned()
            .unwrap_or_default()
    }

    pub fn ty(&self) -> &Type {
        &self.inner.tipe
    }

    /// Return the distribution for which it was packaged
    pub fn distribution(&self) -> Option<&Dist> {
        self.inner.info.distro.as_ref()
    }

    /// Version of the package
    pub fn version(&self) -> &str {
        &self.inner.ver
    }

    pub fn download_url(&self) -> &str {
        &self.inner.url
    }

    pub fn file_name(&self) -> &str {
        self.inner.url.split('/').last().unwrap()
    }

    /// Download package data
    ///
    /// It is required to call this function before calling the `data()` function.
    pub async fn download(&self) -> Result<()> {
        let data = REQWEST
            .get(self.download_url())
            .send()
            .await?
            .bytes()
            .await?;
        *self.inner.data.lock().unwrap() = Data::Package(data.to_vec());
        Ok(())
    }

    /// Return the data of the package.
    ///
    /// It is required to call the `download()` or `set_metadata()` function before calling this.
    /// Otherwise, `None` is returned.
    pub fn data(&self) -> Data {
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

    /// Set the package metadata.
    pub fn set_metadata(&self, metadata: String) {
        *self.inner.data.lock().unwrap() = Data::Metadata(metadata);
    }

    /// Check if metadata is available.
    pub fn is_metadata_available(&self) -> bool {
        matches!(*self.inner.data.lock().unwrap(), Data::Metadata(_))
    }
}

fn split_extention(s: &str) -> Option<Type> {
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

    // `str` is in reverse order, so we try to match it reversely.
    let tipe = match str.as_str() {
        "bed" => Type::Deb,
        "mpr" => Type::Rpm,
        _ => return None,
    };

    Some(tipe)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A shorthand for `Package::detect_package()`
    ///
    /// For testing purpose.
    pub(crate) fn package(p: &str) -> Package {
        Package::detect_package(p, String::new(), p.to_owned(), chrono::DateTime::UNIX_EPOCH)
            .unwrap()
    }

    /// A shorthand for `Package::detect_package()`
    ///
    /// For testing purpose.
    pub(crate) fn package_with_ver(p: &str, v: &str) -> Package {
        Package::detect_package(p, v.to_owned(), p.to_owned(), chrono::DateTime::UNIX_EPOCH)
            .unwrap()
    }

    #[test]
    fn test_package() {
        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "2.0.0");
        assert_eq!(pack.distribution(), Some(&Dist::ubuntu("22.04")));
        assert_eq!(*pack.ty(), Type::Deb);

        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-fedora36.rpm",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "2.0.0");
        assert_eq!(pack.distribution(), Some(&Dist::fedora("36")));
        assert_eq!(*pack.ty(), Type::Rpm);

        let pack = Package::detect_package(
            "caprine_2.56.1_amd64.deb",
            "v2.56.1".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "v2.56.1");
        assert_eq!(pack.distribution(), None);
        assert_eq!(*pack.ty(), Type::Deb);

        let pack = Package::detect_package(
            "ibus-openbangla_3.0.0-opensuse-tumbleweed.rpm",
            "3.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();
        assert_eq!(pack.version(), "3.0.0");
        assert_eq!(pack.distribution(), Some(&Dist::Tumbleweed));
        assert_eq!(*pack.ty(), Type::Rpm);
    }

    #[test]
    fn test_package_change_propagation() {
        let pack = Package::detect_package(
            "OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb",
            "2.0.0".to_owned(),
            String::new(),
            DateTime::UNIX_EPOCH,
        )
        .unwrap();

        assert!(!pack.is_metadata_available());

        let pack2 = pack.clone();
        pack2.set_metadata(String::new());

        assert!(pack.is_metadata_available());
    }

    #[test]
    fn test_split_extension() {
        assert_eq!(
            split_extention("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb"),
            Some(Type::Deb)
        );
        assert_eq!(
            split_extention("OpenBangla-Keyboard_2.0.0-fedora36.rpm"),
            Some(Type::Rpm)
        );
        assert_eq!(split_extention("caprine_2.56.1_amd64.snap"), None);
        assert_eq!(split_extention("deb"), None);
    }
}
