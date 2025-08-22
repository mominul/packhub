use askama::Template;
use sha2::Sha256;
use zstd::encode_all;

use crate::utils::hashsum;

use super::package::RPMPackage;

#[derive(Template)]
#[template(path = "primary.xml")]
struct Primary<'a> {
    packages: &'a [RPMPackage],
}

#[derive(Template)]
#[template(path = "filelists.xml")]
struct FileLists<'a> {
    packages: &'a [RPMPackage],
}

#[derive(Template)]
#[template(path = "other.xml")]
struct Other<'a> {
    packages: &'a [RPMPackage],
}

#[derive(Template)]
#[template(path = "repomd.xml")]
struct RepoMD {
    primary: Metadata,
    filelists: Metadata,
    other: Metadata,
    timestamp: i64,
}

struct Metadata {
    sha256: String,
    open_sha256: String,
    size: usize,
    open_size: usize,
}

pub fn get_primary_index(packages: &[RPMPackage]) -> String {
    let primary = Primary { packages };
    primary.render().unwrap()
}

pub fn get_filelists_index(packages: &[RPMPackage]) -> String {
    let list = FileLists { packages };
    list.render().unwrap()
}

pub fn get_other_index(packages: &[RPMPackage]) -> String {
    let list = Other { packages };
    list.render().unwrap()
}

pub fn get_repomd_index(packages: &[RPMPackage]) -> String {
    let primary = get_primary_index(packages);
    let filelists = get_filelists_index(packages);
    let other = get_other_index(packages);

    // Find the latest date from the list of packages
    let mut timestamp = 0;
    for package in packages {
        if package.pkg_time > timestamp {
            timestamp = package.pkg_time;
        }
    }

    let repomd = RepoMD::create(primary, filelists, other, timestamp);

    repomd.render().unwrap()
}

impl Metadata {
    /// Create the metadata of the `content`.
    fn create(content: String) -> Metadata {
        let data = content.as_bytes();
        let open_size = data.len();
        let open_sha256 = hashsum::<Sha256>(data);
        let compressed = encode_all(data, 0).unwrap();
        let size = compressed.len();
        let sha256 = hashsum::<Sha256>(&compressed);

        Metadata {
            sha256,
            open_sha256,
            size,
            open_size,
        }
    }
}

impl RepoMD {
    fn create(primary: String, filelists: String, other: String, timestamp: i64) -> RepoMD {
        let primary = Metadata::create(primary);
        let filelists = Metadata::create(filelists);
        let other = Metadata::create(other);

        RepoMD {
            primary,
            filelists,
            other,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use chrono::DateTime;
    use insta::assert_snapshot;

    use super::*;
    use crate::package::{Package, tests::package_with_ver};

    #[test]
    fn test_rpm_indices() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora38.rpm", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-fedora38.rpm".to_owned(), DateTime::parse_from_rfc2822("Wed, 8 Nov 2023 16:40:12 +0000").unwrap().into()).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-fedora38.rpm").unwrap();
        package.set_package_data(data);
        let package = RPMPackage::from_package(&package).unwrap();
        let packages = vec![package];

        assert_snapshot!(get_primary_index(&packages));

        assert_snapshot!(get_filelists_index(&packages));

        assert_snapshot!(get_other_index(&packages));

        assert_snapshot!(get_repomd_index(&packages));
    }

    #[test]
    fn test_multiple_arch() {
        let package = package_with_ver("fastfetch-linux-aarch64.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-aarch64.rpm").unwrap();
        package.set_package_data(data);
        let package1 = RPMPackage::from_package(&package).unwrap();

        let package = package_with_ver("fastfetch-linux-amd64.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-amd64.rpm").unwrap();
        package.set_package_data(data);
        let package2 = RPMPackage::from_package(&package).unwrap();

        let package = package_with_ver("fastfetch-linux-armv6l.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-armv6l.rpm").unwrap();
        package.set_package_data(data);
        let package3 = RPMPackage::from_package(&package).unwrap();

        let package = package_with_ver("fastfetch-linux-armv7l.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-armv7l.rpm").unwrap();
        package.set_package_data(data);
        let package4 = RPMPackage::from_package(&package).unwrap();

        let package = package_with_ver("fastfetch-linux-ppc64le.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-ppc64le.rpm").unwrap();
        package.set_package_data(data);
        let package5 = RPMPackage::from_package(&package).unwrap();

        let package = package_with_ver("fastfetch-linux-s390x.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-s390x.rpm").unwrap();
        package.set_package_data(data);
        let package6 = RPMPackage::from_package(&package).unwrap();

        let package = package_with_ver("fastfetch-linux-riscv64.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-riscv64.rpm").unwrap();
        package.set_package_data(data);
        let package7 = RPMPackage::from_package(&package).unwrap();

        let packages = vec![
            package1, package2, package3, package4, package5, package6, package7,
        ];

        assert_snapshot!(get_primary_index(&packages));
        assert_snapshot!(get_filelists_index(&packages));
        assert_snapshot!(get_other_index(&packages));
        assert_snapshot!(get_repomd_index(&packages));
    }
}
