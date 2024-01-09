use anyhow::{Context, Result};
use rpm::{DependencyFlags, FileMode, IndexSignatureTag};
use sha2::Sha256;

use crate::utils::hashsum;

#[derive(Debug)]
struct RPM {
    name: String,
    epoch: u32,
    version: String,
    release: String,
    arch: String,
    vendor: String,
    url: String,
    license: String,
    summary: String,
    description: String,
    group: String,
    build_time: u64,
    build_host: String,
    source: String,
    provides: Vec<Dependency>,
    requires: Vec<Dependency>,
    sha256: String,
    header_start: u64,
    header_end: u64,
    files: Vec<File>,
    pkg_size: usize,
    installed_size: u64,
    archive_size: u64,
}

#[derive(Debug)]
struct Dependency {
    dependency: String,
    version: String,
    condition: String,
}

#[derive(Debug)]
struct File {
    path: String,
    dir: bool,
}

pub fn parse_rpm(mut data: &[u8]) -> Result<RPM> {
    let pkg_size = data.len();
    let rpm = rpm::Package::parse(&mut data)
        .context("Unable to parse the package using rpm parser crate")?;
    let header = rpm.metadata;

    let name = header.get_name()?.to_owned();
    let epoch = header.get_epoch().unwrap_or(0);
    let version = header.get_version()?.to_owned();
    let release = header.get_release()?.to_owned();
    let arch = header.get_arch()?.to_owned();
    let vendor = header.get_vendor()?.to_owned();
    let url = header.get_url()?.to_owned();
    let license = header.get_license()?.to_owned();
    let summary = header.get_summary()?.to_owned();
    let description = header.get_description()?.to_owned();
    let group = header.get_group()?.to_owned();
    let build_time = header.get_build_time()?;
    let build_host = header.get_build_host()?.to_owned();
    let source = header.get_source_rpm()?.to_owned();
    let sha256 = hashsum::<Sha256>(data);
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

    Ok(RPM {
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
    })
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

    use super::*;

    #[test]
    fn test_parser() {
        let data = read("data/OpenBangla-Keyboard_2.0.0-fedora38.rpm").unwrap();
        let parsed = parse_rpm(&data[..]).unwrap();
        assert_debug_snapshot!(parsed);
    }
}
