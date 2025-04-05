use std::{collections::HashMap, io::Write};

use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};
use libflate::gzip::{EncodeOptions, Encoder, HeaderBuilder};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::{
    apt::deb::DebianPackage,
    package::Package,
    utils::{Arch, hashsum},
};

#[derive(Debug)]
pub struct AptIndices {
    packages: HashMap<Arch, Vec<DebianPackage>>,
    date: DateTime<Utc>,
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
struct PackageIndex<'a> {
    packages: &'a [DebianPackage],
}

struct Files {
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
    size: usize,
    path: String,
}

impl AptIndices {
    pub fn new(packages: &[Package]) -> Result<AptIndices> {
        let mut debian: HashMap<Arch, Vec<DebianPackage>> = HashMap::new();
        // Find the latest date from the list of packages
        let mut date = DateTime::UNIX_EPOCH;
        for package in packages {
            if *package.creation_date() > date {
                date = *package.creation_date();
            }

            match DebianPackage::from_package(package) {
                Ok(deb) => {
                    if let Some(arch) = deb.get_arch() {
                        debian.entry(arch).or_default().push(deb);
                    } else {
                        tracing::error!(
                            "Debian package architecture not found for package: {:?}",
                            package
                        );
                        continue;
                    }
                }
                Err(e) => {
                    tracing::error!("Error occurred when extracting debian control data: {e}");
                    continue;
                }
            }
        }
        Ok(AptIndices {
            packages: debian,
            date,
        })
    }

    pub fn get_package_index(&self, arch: &Arch) -> String {
        let index = PackageIndex {
            packages: self.packages.get(arch).unwrap(),
        };
        index.render().unwrap().trim().to_owned()
    }

    pub fn get_release_index(&self) -> String {
        let name = ". stable";
        let date = self.date.to_rfc2822();

        let mut files = vec![];

        for arch in self.packages.keys() {
            let packages = self.get_package_index(arch);
            let packages = packages.as_bytes();
            let packages_gz = gzip_compression(packages);

            files.extend([
                Files {
                    sha256: hashsum::<Sha256>(packages),
                    size: packages.len(),
                    path: format!("main/binary-{}/Packages", arch),
                    md5: hashsum::<Md5>(packages),
                    sha1: hashsum::<Sha1>(packages),
                    sha512: hashsum::<Sha512>(packages),
                },
                Files {
                    sha256: hashsum::<Sha256>(&packages_gz),
                    size: packages_gz.len(),
                    path: format!("main/binary-{}/Packages.gz", arch),
                    md5: hashsum::<Md5>(&packages_gz),
                    sha1: hashsum::<Sha1>(&packages_gz),
                    sha512: hashsum::<Sha512>(&packages_gz),
                },
            ]);
        }

        // Sort the files
        files.sort_by(|a, b| a.path.cmp(&b.path));

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

    gzip.into_result().unwrap()
}

#[cfg(test)]
mod tests {
    use std::fs::{self, read};

    use chrono::DateTime;
    use insta::assert_snapshot;

    use super::*;
    use crate::package::tests::package_with_ver;

    #[test]
    fn test_apt_indices() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned(), DateTime::parse_from_rfc2822("Wed, 8 Nov 2023 16:40:12 +0000").unwrap().into()).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();
        package.set_package_data(data);

        let packages = vec![package];

        let indices = AptIndices::new(&packages).unwrap();

        // Packages
        let packages = indices.get_package_index(&Arch::Amd64);
        assert_snapshot!(packages);

        // Release
        let release = indices.get_release_index();
        assert_snapshot!(release);
    }

    #[test]
    fn test_multiple_packages() {
        let package1 = package_with_ver("fcitx-openbangla_3.0.0.deb", "3.0.0");
        let data = fs::read("data/fcitx-openbangla_3.0.0.deb").unwrap();
        package1.set_package_data(data);

        let package2 = package_with_ver("ibus-openbangla_3.0.0.deb", "3.0.0");
        let data = fs::read("data/ibus-openbangla_3.0.0.deb").unwrap();
        package2.set_package_data(data);

        let packages = vec![package1, package2];

        let indices = AptIndices::new(&packages).unwrap();

        // Packages
        let packages = indices.get_package_index(&Arch::Amd64);
        assert_snapshot!(packages);
        assert_eq!(packages.as_bytes().len(), 2729);
        let packages_gz = gzip_compression(packages.as_bytes());
        assert_eq!(packages_gz.len(), 1105);

        // Release
        let release = indices.get_release_index();
        assert_snapshot!(release);
    }

    #[test]
    fn test_multiple_architectures() {
        let package1 = package_with_ver("fastfetch-linux-aarch64.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-aarch64.deb").unwrap();
        package1.set_package_data(data);

        let package2 = package_with_ver("fastfetch-linux-amd64.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-amd64.deb").unwrap();
        package2.set_package_data(data);

        let package3 = package_with_ver("fastfetch-linux-armv6l.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-armv6l.deb").unwrap();
        package3.set_package_data(data);

        let package4 = package_with_ver("fastfetch-linux-armv7l.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-armv7l.deb").unwrap();
        package4.set_package_data(data);

        let package5 = package_with_ver("fastfetch-linux-ppc64le.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-ppc64le.deb").unwrap();
        package5.set_package_data(data);

        let package6 = package_with_ver("fastfetch-linux-riscv64.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-riscv64.deb").unwrap();
        package6.set_package_data(data);

        let package7 = package_with_ver("fastfetch-linux-s390x.deb", "2.40.3");
        let data = fs::read("data/fastfetch-linux-s390x.deb").unwrap();
        package7.set_package_data(data);

        let packages = [
            package1, package2, package3, package4, package5, package6, package7,
        ];

        let indices = AptIndices::new(&packages).unwrap();

        // Release
        let release = indices.get_release_index();
        assert_snapshot!(release);
    }
}
