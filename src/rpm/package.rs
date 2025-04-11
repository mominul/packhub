use anyhow::{Context, Result, bail};
use rpm::{DependencyFlags, FileMode, IndexSignatureTag};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use sha2::Sha256;

use crate::{
    package::{Data, Package},
    utils::hashsum,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RPMPackage {
    pub name: String,
    pub epoch: u32,
    pub version: String,
    pub release: String,
    pub arch: String,
    pub vendor: Option<String>,
    pub url: Option<String>,
    pub license: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub group: Option<String>,
    pub build_time: u64,
    pub build_host: Option<String>,
    pub source: Option<String>,
    pub provides: Vec<Dependency>,
    pub requires: Vec<Dependency>,
    pub sha256: String,
    pub header_start: u64,
    pub header_end: u64,
    pub files: Vec<File>,
    pub pkg_size: usize,
    pub installed_size: u64,
    pub archive_size: u64,
    pub location: String,
    pub pkg_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub dependency: String,
    pub version: String,
    pub condition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub dir: bool,
}

impl RPMPackage {
    /// Parse the package and return the RPM package.
    ///
    /// Also sets the metadata to the package.
    pub fn from_package(package: &Package) -> Result<RPMPackage> {
        // If the metadata is already available, then build the RPMPackage from it
        if let Data::Metadata(metadata) = package.data() {
            let rpm: RPMPackage = from_str(&metadata).context("Error while loading RPMPackage from saved Package metadata")?;
            return Ok(rpm);
        }

        let Data::Package(data) = package.data() else {
            bail!("Data isn't loaded in package");
        };

        let mut data = data.as_slice();
        // Calculate these before the data slice is mutated
        let pkg_size = data.len();
        let sha256 = hashsum::<Sha256>(data);

        let rpm = rpm::Package::parse(&mut data)
            .context("Unable to parse the package using rpm parser crate")?;
        let header = rpm.metadata;

        let name = header.get_name()?.to_owned();
        let epoch = header.get_epoch().unwrap_or(0);
        let version = header.get_version()?.to_owned();
        let release = header.get_release()?.to_owned();
        let arch = header.get_arch()?.to_owned();
        let vendor = header.get_vendor().ok().map(|v| v.to_owned());
        let url = header.get_url().ok().map(|v| v.to_owned());
        let license = header.get_license().ok().map(|v| v.to_owned());
        let summary = header.get_summary().ok().map(|v| v.to_owned());
        let description = header.get_description().ok().map(|v| v.to_owned());
        let group = header.get_group().ok().map(|v| v.to_owned());
        let build_time = header.get_build_time()?;
        let build_host = header.get_build_host().ok().map(|v| v.to_owned());
        let source = header.get_source_rpm().ok().map(|v| v.to_owned());
        let range = header.get_package_segment_offsets();
        let header_start = range.header;
        let header_end = range.payload;
        let installed_size = header.get_installed_size()?;
        let archive_size = header
            .signature
            .get_entry_data_as_u64(IndexSignatureTag::RPMSIGTAG_LONGARCHIVESIZE)
            .or_else(|_e| {
                header
                    .signature
                    .get_entry_data_as_u32(IndexSignatureTag::RPMSIGTAG_PAYLOADSIZE)
                    .map(|v| v as u64)
            })?;

        let provides: Vec<Dependency> = header
            .get_provides()?
            .into_iter()
            .map(|i| Dependency {
                dependency: i.name,
                version: i.version,
                condition: flag_to_condition(i.flags),
            })
            .collect();

        let requires: Vec<Dependency> = header
            .get_requires()?
            .into_iter()
            .filter(|i| !i.flags.contains(DependencyFlags::RPMLIB))
            .filter(|i| !i.flags.contains(DependencyFlags::CONFIG))
            .map(|i| Dependency {
                dependency: i.name,
                version: i.version,
                condition: flag_to_condition(i.flags),
            })
            .collect();

        let files: Vec<File> = header
            .get_file_entries()?
            .into_iter()
            .map(|i| File {
                path: i.path.to_string_lossy().to_string(),
                dir: matches!(i.mode, FileMode::Dir { .. }),
            })
            .collect();

        let location = format!("package/{}/{}", package.version(), package.file_name());
        let pkg_time = package.creation_date().timestamp();

        let rpm = RPMPackage {
            name,
            epoch,
            version,
            release,
            arch,
            vendor,
            url,
            license,
            summary,
            description,
            group,
            build_time,
            build_host,
            source,
            provides,
            requires,
            sha256,
            header_start,
            header_end,
            files,
            pkg_size,
            installed_size,
            archive_size,
            location,
            pkg_time,
        };

        // Set the matadata to the package
        let metadata = to_string(&rpm)?;
        package.set_metadata(metadata);

        Ok(rpm)
    }
}

fn flag_to_condition(flags: DependencyFlags) -> String {
    if flags.contains(DependencyFlags::GE) {
        "GE".to_owned()
    } else if flags.contains(DependencyFlags::LE) {
        "LE".to_owned()
    } else if flags.contains(DependencyFlags::EQUAL) {
        "EQ".to_owned()
    } else {
        "".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use insta::assert_debug_snapshot;

    use crate::package::tests::package_with_ver;
    use super::*;

    #[test]
    fn test_parser() {
        let package = package_with_ver("OpenBangla-Keyboard_2.0.0-fedora38.rpm", "2.0.0");
        let data = read("data/OpenBangla-Keyboard_2.0.0-fedora38.rpm").unwrap();
        package.set_package_data(data);
        let parsed = RPMPackage::from_package(&package).unwrap();
        assert_debug_snapshot!(parsed);

        let package = package_with_ver("fastfetch-linux-amd64.rpm", "2.40.3");
        let data = read("data/fastfetch-linux-amd64.rpm").unwrap();
        package.set_package_data(data);
        let parsed = RPMPackage::from_package(&package).unwrap();
        assert_debug_snapshot!(parsed);
    }

    #[test]
    #[should_panic]
    fn test_package_without_data() {
        let package = package_with_ver("OpenBangla-Keyboard_2.0.0-fedora38.rpm", "2.0.0");

        RPMPackage::from_package(&package).unwrap();
    }

    #[test]
    fn test_loading_from_metadata() {
        let package = package_with_ver("OpenBangla-Keyboard_2.0.0-fedora38.rpm", "2.0.0");
        let data = read("data/OpenBangla-Keyboard_2.0.0-fedora38.rpm").unwrap();
        package.set_package_data(data);
        let _ = RPMPackage::from_package(&package).unwrap();

        // The package data should have been replaced by the metadata
        assert!(matches!(package.data(), Data::Metadata(_)));

        let _ = RPMPackage::from_package(&package).unwrap();
    }
}
