use crate::{
    package::{Dist, Package},
    platform::match_ubuntu_for_apt,
};

pub(crate) fn select_package_ubuntu<'p>(from: &'p Vec<Package>, apt: &str) -> &'p Package {
    from.iter()
        .filter(|p| p.for_ubuntu())
        .filter(|p| match_ubuntu_for_apt(apt) == *p.distribution())
        .nth(0)
        .unwrap()
}

pub(crate) fn select_package<'p>(from: &'p [Package], dist: Dist) -> &'p Package {
    from.iter()
        .filter(|p| dist == *p.distribution())
        .nth(0)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use lenient_semver::parse;

    use super::*;

    fn openbangla_keyboard_packages() -> Vec<Package> {
        [
            // TODO: Package::detect_package("OpenBangla-Keyboard_2.0.0-archlinux.pkg.tar.zst",  String::new()).unwrap(),
            package("OpenBangla-Keyboard_2.0.0-debian10-buster.deb"),
            package("OpenBangla-Keyboard_2.0.0-debian11.deb"),
            package("OpenBangla-Keyboard_2.0.0-debian9-stretch.deb"),
            package("OpenBangla-Keyboard_2.0.0-fedora29.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora30.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora31.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora32.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora33.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora34.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora35.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora36.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora37.rpm"),
            package("OpenBangla-Keyboard_2.0.0-fedora38.rpm"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu18.04.deb"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu19.10.deb"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu21.04.deb"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb"),
        ]
        .into()
    }

    /// A shorthand for `Package::detect_package()`
    fn package(p: &str) -> Package {
        Package::detect_package(
            p,
            String::new(),
            String::new(),
            chrono::DateTime::UNIX_EPOCH,
        )
        .unwrap()
    }

    #[test]
    fn test_package_selection_ubuntu() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            *select_package_ubuntu(&packages, "1.6.14"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu18.04.deb")
        );
        assert_eq!(
            *select_package_ubuntu(&packages, "2.0.9"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb")
        );
        assert_eq!(
            *select_package_ubuntu(&packages, "2.4.8"),
            package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb")
        );
    }

    #[test]
    fn test_package_selection_fedora() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            *select_package(&packages, Dist::Fedora(parse("38").ok())),
            package("OpenBangla-Keyboard_2.0.0-fedora38.rpm")
        );
    }
}
