use askama::Template;

use super::package::{Dependency, RPMPackage};

#[derive(Debug, Template)]
#[template(path = "primary.xml")]
struct Primary {
    packages: Vec<RPMPackage>,
}

#[derive(Debug, Template)]
#[template(path = "filelists.xml")]
struct FileLists {
    packages: Vec<RPMPackage>,
}

#[derive(Debug, Template)]
#[template(path = "other.xml")]
struct Other {
    packages: Vec<RPMPackage>,
}

pub fn get_primary_index(packages: Vec<RPMPackage>) -> String {
    let primary = Primary { packages };
    primary.render().unwrap()
}

pub fn get_filelists_index(packages: Vec<RPMPackage>) -> String {
    let list = FileLists { packages };
    list.render().unwrap()
}

pub fn get_other_index(packages: Vec<RPMPackage>) -> String {
    let list = Other { packages };
    list.render().unwrap()
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

        assert_snapshot!(get_primary_index(packages.clone()));

        assert_snapshot!(get_filelists_index(packages.clone()));

        assert_snapshot!(get_other_index(packages));
    }
}
