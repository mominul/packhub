use std::fmt::Write as _;
use std::io::Write as _;

use chrono::Utc;
use libflate::gzip::Encoder;
use askama::Template;

use crate::{detect::Package, deb::DebAnalyzer};

struct AptIndices<'a> {
    data: &'a [u8],
    package: &'a Package,
}

#[derive(Template)]
#[template(path = "RELEASE")] 
struct ReleaseIndex {
    date: String,
    files: Vec<Files>,
}

struct Files {
    sum256: String,
    size: usize,
    path: String, 
}

impl<'a> AptIndices<'a> {
    fn new(package: &'a Package, data: &'a [u8]) -> Self {
        AptIndices { data, package }
    }

    fn get_package_index(&self) -> String {
        let deb = DebAnalyzer::new(self.data);
        let mut control_data = deb.get_control_data().trim_end().to_string();
        write!(&mut control_data, "\nFilename: pool/stable/{}/{}\n\n", self.package.version(), self.package.file_name()).unwrap();
        control_data
    }

    fn get_release_index(&self) -> String {
        let date = Utc::now().to_rfc2822();

        let packages = self.get_package_index();
        let packages = packages.as_bytes();

        let packages_gz = gzip_compression(packages);

        let files = vec![
            Files { sum256: sha256::digest(packages), size: packages.len(), path: "main/binary-amd64/Packages".to_string() },
            Files { sum256: sha256::digest(&packages_gz), size: packages_gz.len(), path: "main/binary-amd64/Packages.gz".to_string() }
        ];

        let index = ReleaseIndex { date, files };

        index.render().unwrap()
    }
}

fn gzip_compression(data: &[u8]) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new()).unwrap();
    encoder.write_all(data).unwrap();

    let gzip = encoder.finish();
    let gzip = gzip.into_result().unwrap();

    gzip
}

#[cfg(test)]
mod tests {
    use std::fs::{write, self};

    use super::*;

    #[test]
    fn test_apt_indices() {
        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned()).unwrap();
        let data = fs::read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();

        let indices = AptIndices::new(&package, &data);

        assert_eq!(indices.get_package_index(), "Architecture: amd64\nDepends: ibus (>= 1.5.1), libc6 (>= 2.29), libgcc-s1 (>= 4.2), libglib2.0-0 (>= 2.12.0), libibus-1.0-5 (>= 1.5.1), libqt5core5a (>= 5.12.2), libqt5gui5 (>= 5.0.2) | libqt5gui5-gles (>= 5.0.2), libqt5network5 (>= 5.0.2), libqt5widgets5 (>= 5.0.2), libstdc++6 (>= 5.2), libzstd1 (>= 1.3.2)\nDescription: OpenSource Bengali input method\n OpenBangla Keyboard is an OpenSource, Unicode compliant Bengali Input Method for GNU/Linux systems. It&apos;s a full fledged Bengali input method with typing automation tools, includes many famous typing methods such as Avro Phonetic, Probhat, Munir Optima, National (Jatiya) etc.\n .\n Most of the features of Avro Keyboard are present in OpenBangla Keyboard. So Avro Keyboard users will feel at home with OpenBangla Keyboard in Linux.\n .\nHomepage: https://openbangla.github.io/\nMaintainer: OpenBangla Team <openbanglateam@gmail.com>\nPackage: openbangla-keyboard\nPriority: optional\nSection: utils\nVersion: 2.0.0\nInstalled-Size: 12263\nFilename: pool/stable/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb\n\n");
    }
}
