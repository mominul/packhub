use std::io::Read;

use libflate::gzip::Decoder;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::detect::Arch;

static ARCH: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Architecture: (\w+)"#).unwrap());
static PACKAGE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Package: (.+)"#).unwrap());

/// Debian package (.deb) file analyzer
pub struct DebAnalyzer {
    control: String,
}

impl DebAnalyzer {
    pub fn new(data: &[u8]) -> Self {
        let control = read_control_file(data);

        Self { control }
    }

    /// Get the control data from the package.
    pub fn get_control_data(&self) -> &str {
        &self.control
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
    use super::*;

    #[test]
    fn test_deb_analyzer() {
        let data = include_bytes!("../../data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb");
        let deb = DebAnalyzer::new(data);
        assert_eq!(deb.get_arch(), Ok(Arch::Amd64));
        assert_eq!(deb.get_package(), "openbangla-keyboard");
    }
}
