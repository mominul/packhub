use std::io::Read;

use anyhow::{bail, Result};
use libflate::gzip::Decoder;
use md5::Md5;
use mongodb::bson::{from_slice, to_vec};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::{
    package::{Data, Package},
    utils::{hashsum, Arch},
};

static ARCH: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Architecture: (\w+)"#).unwrap());
static PACKAGE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Package: (.+)"#).unwrap());

/// Debian package (.deb)
#[derive(Serialize, Deserialize)]
pub struct DebianPackage {
    pub control: String,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub size: usize,
    pub filename: String,
}

impl DebianPackage {
    /// Create a new Debian package from a package.
    ///
    /// Also sets metadata of the package.
    pub fn from_package(package: &Package) -> Result<Self> {
        // Create the debian package from the metadata if it is present.
        if let Data::Metadata(metadata) = package.data() {
            let package: DebianPackage = from_slice(&metadata)?;

            return Ok(package);
        }

        let Data::Package(data) = package.data() else {
            bail!("Package data is not available");
        };

        let control = read_control_file(&data).trim_end().to_owned();
        let filename = format!("pool/stable/{}/{}", package.version(), package.file_name());

        let size = data.len();
        let md5 = hashsum::<Md5>(&data);
        let sha1 = hashsum::<Sha1>(&data);
        let sha256 = hashsum::<Sha256>(&data);
        let sha512 = hashsum::<Sha512>(&data);

        let deb = Self {
            control,
            md5,
            sha1,
            sha256,
            sha512,
            size,
            filename,
        };

        let metadata = to_vec(&deb)?;
        package.set_metadata(metadata);

        Ok(deb)
    }

    /// Get the architecture for which the package is built for.
    pub fn get_arch(&self) -> Result<Arch, ()> {
        ARCH.captures(&self.control)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse()
    }

    /// Get the package name.
    pub fn get_package(&self) -> &str {
        PACKAGE
            .captures(&self.control)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
    }
}

fn read_control_file(data: &[u8]) -> String {
    let mut archive = ar::Archive::new(data);

    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result.unwrap();
        let header = entry.header();
        let name = String::from_utf8_lossy(header.identifier());
        if name == "control.tar.gz" {
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();

            // Un-compress the control.tar.gz
            let mut decoder = Decoder::new(&data[..]).unwrap();
            let mut data = Vec::new();
            decoder.read_to_end(&mut data).unwrap();

            // Read the control.tar archive
            let mut archive = tar::Archive::new(&data[..]);
            for entry in archive.entries().unwrap() {
                let mut entry = entry.unwrap();

                let path = entry.path().unwrap();
                let path = path.to_str().unwrap();

                if path == "./control" {
                    let mut control = String::new();
                    entry.read_to_string(&mut control).unwrap();

                    return control;
                }
            }
        }
    }
    return String::new();
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use chrono::DateTime;

    use super::*;

    #[test]
    fn test_parsing() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned(), DateTime::UNIX_EPOCH).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();
        package.set_package_data(data);

        let deb = DebianPackage::from_package(&package).unwrap();
        assert_eq!(deb.get_arch(), Ok(Arch::Amd64));
        assert_eq!(deb.get_package(), "openbangla-keyboard");
    }

    #[test]
    #[should_panic]
    fn test_without_data() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned(), DateTime::UNIX_EPOCH).unwrap();

        let _ = DebianPackage::from_package(&package).unwrap();
    }

    #[test]
    fn test_loading_from_metadata() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned(), DateTime::UNIX_EPOCH).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();
        package.set_package_data(data);

        let _ = DebianPackage::from_package(&package).unwrap();

        // The package data should have been replaced by the metadata
        assert!(matches!(package.data(), Data::Metadata(_)));

        let _ = DebianPackage::from_package(&package).unwrap();
    }
}
