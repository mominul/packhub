use std::io::Write;

use anyhow::Result;
use askama::Template;
use libflate::gzip::{EncodeOptions, Encoder, HeaderBuilder};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::{apt::deb::DebianPackage, package::Package, utils::hashsum};

pub struct AptIndices<'a> {
    packages: &'a [Package],
}

#[derive(Template)]
#[template(path = "Release")]
struct ReleaseIndex<'a> {
    origin: &'a str,
    label: &'a str,
    date: String,
    files: Vec<Files>,
}

#[derive(Template)]
#[template(path = "Packages")]
struct PackageIndex {
    packages: Vec<DebianPackage>,
}

struct Files {
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    size: usize,
    path: String,
}

impl<'a> AptIndices<'a> {
    pub fn new(packages: &'a [Package]) -> Result<AptIndices<'a>> {
        Ok(AptIndices { packages })
    }

    pub fn get_package_index(&self) -> String {
        let packages = self
            .packages
            .iter()
            .map(|package| DebianPackage::from_package(package))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let index = PackageIndex { packages };
        index.render().unwrap().trim().to_owned()
    }

    pub fn get_release_index(&self) -> String {
        // Find the latest date from the list of packages
        let mut date = chrono::DateTime::UNIX_EPOCH;
        for package in self.packages {
            if *package.creation_date() > date {
                date = *package.creation_date();
            }
        }

        let date = date.to_rfc2822();

        let packages = self.get_package_index();
        let packages = packages.as_bytes();

        let name = ". stable"; //format!("{} stable", self.deb.get_package());

        let packages_gz = gzip_compression(packages);

        let files = vec![
            Files {
                sha256: hashsum::<Sha256>(packages),
                size: packages.len(),
                path: "main/binary-amd64/Packages".to_string(),
                md5: hashsum::<Md5>(packages),
                sha1: hashsum::<Sha1>(packages),
                sha512: hashsum::<Sha512>(packages),
            },
            Files {
                sha256: hashsum::<Sha256>(&packages_gz),
                size: packages_gz.len(),
                path: "main/binary-amd64/Packages.gz".to_string(),
                md5: hashsum::<Md5>(&packages_gz),
                sha1: hashsum::<Sha1>(&packages_gz),
                sha512: hashsum::<Sha512>(&packages_gz),
            },
        ];

        let index = ReleaseIndex {
            date,
            files,
            origin: name,
            label: name,
        };

        index.render().unwrap()
    }
}

pub fn gzip_compression(data: &[u8]) -> Vec<u8> {
    let header = HeaderBuilder::new().modification_time(0).finish();
    let options = EncodeOptions::new().header(header);
    let mut encoder = Encoder::with_options(Vec::new(), options).unwrap();
    encoder.write_all(data).unwrap();

    let gzip = encoder.finish();
    let gzip = gzip.into_result().unwrap();

    gzip
}

#[cfg(test)]
mod tests {
    use std::fs::{self, read};

    use chrono::DateTime;
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_apt_indices() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned(), DateTime::parse_from_rfc2822("Wed, 8 Nov 2023 16:40:12 +0000").unwrap().into(), DateTime::UNIX_EPOCH).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();
        package.set_package_data(data);

        let packages = vec![package];

        let indices = AptIndices::new(&packages).unwrap();

        // Packages
        let packages = indices.get_package_index();
        assert_snapshot!(packages);

        // Release
        let release = indices.get_release_index();
        assert_snapshot!(release);
    }

    #[test]
    fn test_multiple_packages() {
        let package1 = Package::detect_package("fcitx-openbangla_3.0.0.deb", "3.0.0".to_owned(), "https://github.com/mominul/pack-exp2/releases/download/3.0.0/fcitx-openbangla_3.0.0.deb".to_owned(), DateTime::UNIX_EPOCH, DateTime::UNIX_EPOCH).unwrap();
        let data = fs::read("data/fcitx-openbangla_3.0.0.deb").unwrap();
        package1.set_package_data(data);

        let package2 = Package::detect_package("ibus-openbangla_3.0.0.deb", "3.0.0".to_owned(), "https://github.com/mominul/pack-exp2/releases/download/3.0.0/ibus-openbangla_3.0.0.deb".to_owned(), DateTime::UNIX_EPOCH, DateTime::UNIX_EPOCH).unwrap();
        let data = fs::read("data/ibus-openbangla_3.0.0.deb").unwrap();
        package2.set_package_data(data);

        let packages = vec![package1, package2];

        let indices = AptIndices::new(&packages).unwrap();

        // Packages
        let packages = indices.get_package_index();
        assert_snapshot!(packages);
        assert_eq!(packages.as_bytes().len(), 2729);
        let packages_gz = gzip_compression(packages.as_bytes());
        assert_eq!(packages_gz.len(), 1105);

        // Release
        let release = indices.get_release_index();
        assert_snapshot!(release);
    }
}
