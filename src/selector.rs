use crate::{detect::Package, platform::match_ubuntu_for_apt};

pub(crate) fn select_package_ubuntu<'p>(from: &'p Vec<Package>, apt: &str) -> &'p Package {
    from.iter()
        .filter(|p| p.for_ubuntu())
        .filter(|p| match_ubuntu_for_apt(apt) == *p.distribution())
        .nth(0)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::ignore = "becomes unreadable"]
    fn openbangla_keyboard_packages() -> Vec<Package> {
        [
            // TODO: Package::detect_package("OpenBangla-Keyboard_2.0.0-archlinux.pkg.tar.zst",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-debian10-buster.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-debian11.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-debian9-stretch.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora29.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora30.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora31.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora32.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora33.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora34.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora35.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-fedora36.rpm",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu18.04.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu19.10.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu21.04.deb",  String::new()).unwrap(),
            Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb",  String::new()).unwrap(),
        ].into()
    }

    fn package(p: &str) -> Package {
        Package::detect_package(p,  String::new()).unwrap()
    }

    #[test]
    fn test_package_selection_ubuntu() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(*select_package_ubuntu(&packages, "1.6.14"), package("OpenBangla-Keyboard_2.0.0-ubuntu18.04.deb"));
        assert_eq!(*select_package_ubuntu(&packages, "2.0.9"), package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb"));
        assert_eq!(*select_package_ubuntu(&packages, "2.4.8"), package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb"));
    }
}
