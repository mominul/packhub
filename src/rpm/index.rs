use askama::Template;
use chrono::Utc;
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

    let repomd = RepoMD::create(primary, filelists, other);

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

        Metadata { sha256, open_sha256, size, open_size }
    }
}

impl RepoMD {
    fn create(primary: String, filelists: String, other: String) -> RepoMD {
        let primary = Metadata::create(primary);
        let filelists = Metadata::create(filelists);
        let other = Metadata::create(other);
        let timestamp = Utc::now().timestamp();


        RepoMD { primary, filelists, other, timestamp }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use chrono::DateTime;
    use insta::assert_snapshot;

    use crate::package::Package;

    use super::*;

    #[test]
    fn test_rpm_indices() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora38.rpm", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-fedora38.rpm".to_owned(), DateTime::UNIX_EPOCH).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-fedora38.rpm").unwrap();
        package.set_data(data);
        let package = RPMPackage::from_package(&package).unwrap();
        let packages = vec![package];

        assert_snapshot!(get_primary_index(&packages));

        assert_snapshot!(get_filelists_index(&packages));

        assert_snapshot!(get_other_index(&packages));

        let repomd = get_repomd_index(&packages);
        insta::with_settings!({filters => vec![
            // Date is a changing value, so replace it with a hardcoded value.
            (r"<timestamp>.+<\/timestamp>", r"<timestamp>TIMESTAMP<\/timestamp>"),
            (r"<revision>.+<\/revision>", r"<revision>TIMESTAMP<\/revision>"),
        ]}, {
            assert_snapshot!(repomd);
        });
    }
}
