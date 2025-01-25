use std::io::Read;
use std::sync::LazyLock;

use anyhow::{bail, Context, Result};
use libflate::gzip::Decoder;
use md5::Md5;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::{
    package::{Data, Package},
    utils::{hashsum, Arch},
};

static ARCH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Architecture: (\w+)"#).unwrap());
static PACKAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"Package: (.+)"#).unwrap());

/// Debian package (.deb)
#[derive(Serialize, Deserialize, Debug)]
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
            let package: DebianPackage = from_str(&metadata)?;

            return Ok(package);
        }

        let Data::Package(data) = package.data() else {
            bail!("Package data is not available");
        };

        let control = read_control_file(&data)
            .context("Error occurred while parsing the debian control file from package")?
            .trim_end()
            .to_owned();
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

        let metadata = to_string(&deb)?;
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

fn read_control_file(data: &[u8]) -> Result<String> {
    let mut archive = ar::Archive::new(data);

    while let Some(entry_result) = archive.next_entry() {
        let mut entry = entry_result?;
        let header = entry.header();
        let name = String::from_utf8_lossy(header.identifier());
        if name == "control.tar.gz" {
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;

            // Un-compress the control.tar.gz
            let mut decoder = Decoder::new(&data[..])?;
            let mut data = Vec::new();
            decoder.read_to_end(&mut data)?;

            // Read the control.tar archive
            let mut archive = tar::Archive::new(&data[..]);
            for entry in archive.entries()? {
                let mut entry = entry?;

                let path = entry.path()?;
                let path = path.to_str().unwrap();

                if path == "./control" {
                    let mut control = String::new();
                    entry.read_to_string(&mut control)?;

                    return Ok(control);
                }
            }
        }
    }

    bail!("Control file not found");
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
