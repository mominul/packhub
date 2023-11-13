use std::io::Write;

use chrono::Utc;
use libflate::gzip::{Encoder, HeaderBuilder, EncodeOptions};
use askama::Template;

use crate::{detect::Package, deb::DebAnalyzer};

pub struct AptIndices<'a> {
    data: &'a [u8],
    package: &'a Package,
    deb: DebAnalyzer,
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
    control: &'a str,
    sum256: String,
    size: usize,
    filename: String,
}

struct Files {
    sum256: String,
    size: usize,
    path: String, 
}

impl<'a> AptIndices<'a> {
    pub fn new(package: &'a Package, data: &'a [u8]) -> Self {
        let deb = DebAnalyzer::new(data);
        AptIndices { data, package, deb }
    }

    pub fn get_package_index(&self) -> String {
        let control = self.deb.get_control_data().trim_end();
        let filename= format!("pool/stable/{}/{}", self.package.version(), self.package.file_name());
        let size = self.data.len();
        let sum256 = sha256::digest(self.data);

        let index = PackageIndex { control, sum256, size, filename };
        index.render().unwrap()
    }

    pub fn get_release_index(&self) -> String {
        let date = Utc::now().to_rfc2822();

        let packages = self.get_package_index();
        let packages = packages.as_bytes();

        let name = self.deb.get_package();

        let packages_gz = gzip_compression(packages);

        let files = vec![
            Files { sum256: sha256::digest(packages), size: packages.len(), path: "main/binary-amd64/Packages".to_string() },
            Files { sum256: sha256::digest(&packages_gz), size: packages_gz.len(), path: "main/binary-amd64/Packages.gz".to_string() }
        ];

        let index = ReleaseIndex { date, files, origin: name, label: name };

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
    use std::fs;

    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_apt_indices() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned()).unwrap();
        let data = fs::read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();

        let indices = AptIndices::new(&package, &data);

        // Packages
        let packages = indices.get_package_index();
        assert_snapshot!(packages);
        
        // Release
        let release = indices.get_release_index();
        insta::with_settings!({filters => vec![
            // Date is a changing value, so replace it with a hardcoded value.
            (r"Date: .+", "Date: [DATE]"),
        ]}, {
            assert_snapshot!(release);
        });
    }
}
