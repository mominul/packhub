use crate::{package::Package, utils::Dist};

pub(crate) fn select_packages(from: &[Package], dist: Dist) -> Vec<&Package> {
    let mut packages = Vec::new();

    // Filter out the packages that are not for the distribution.
    for package in from {
        if package.ty().matches_distribution(&dist) {
            packages.push(package);
        }
    }

    // Find matches for the distribution.
    let mut selective = Vec::new();

    // Loosely match the distribution (without regarding the distribution version).
    for package in packages.iter() {
        if let Some(pack_dist) = package.distribution() {
            if dist.matches_distribution(pack_dist) {
                selective.push(*package);
            }
        }
    }

    // Search for the exact distribution version match.
    // TODO: Handle the case when there is no exact version match
    //       but lower or higher version match is available.
    if !selective.is_empty() {
        let mut exact = Vec::new();

        for package in selective.iter() {
            if Some(&dist) == package.distribution().as_ref() {
                exact.push(*package);
            }
        }

        // If we have exact match packages, then return them.
        if !exact.is_empty() {
            return exact;
        }

        // We have no exact match, so return the selective packages.
        return selective;
    }

    packages
}

#[cfg(test)]
mod tests {
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

    fn multiple_packages() -> Vec<Package> {
        [
            package("fcitx-openbangla_3.0.0.deb"),
            package("ibus-openbangla_3.0.0.deb"),
            package("fcitx-openbangla_3.0.0-fedora.rpm"),
            package("ibus-openbangla_3.0.0-fedora.rpm"),
            package("ibus-openbangla_3.0.0-opensuse-tumbleweed.rpm"),
            package("fcitx-openbangla_3.0.0-opensuse-tumbleweed.rpm"),
        ]
        .into()
    }

    /// A shorthand for `Package::detect_package()`
    fn package(p: &str) -> Package {
        Package::detect_package(p, String::new(), p.to_owned(), chrono::DateTime::UNIX_EPOCH)
            .unwrap()
    }

    #[test]
    fn test_package_selection_ubuntu() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            select_packages(&packages, Dist::Ubuntu(Some("18.04".to_owned()))),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu18.04.deb")]
        );
        assert_eq!(
            select_packages(&packages, Dist::Ubuntu(Some("20.04".to_owned()))),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb")]
        );
        assert_eq!(
            select_packages(&packages, Dist::Ubuntu(Some("22.04".to_owned()))),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb")]
        );
    }

    #[test]
    fn test_package_selection_fedora() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            select_packages(&packages, Dist::Fedora(Some("38".to_owned()))),
            vec![&package("OpenBangla-Keyboard_2.0.0-fedora38.rpm")]
        );
    }

    #[test]
    fn test_package_selection_debian() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            select_packages(&packages, Dist::Debian(Some("11".to_owned()))),
            vec![&package("OpenBangla-Keyboard_2.0.0-debian11.deb")]
        );
    }

    #[test]
    fn test_multiple_package_selection() {
        let packages = multiple_packages();

        assert_eq!(
            select_packages(&packages, Dist::Ubuntu(Some("22.04".to_owned()))),
            vec![
                &package("fcitx-openbangla_3.0.0.deb"),
                &package("ibus-openbangla_3.0.0.deb")
            ]
        );

        assert_eq!(
            select_packages(&packages, Dist::Fedora(Some("39".to_owned()))),
            vec![
                &package("fcitx-openbangla_3.0.0-fedora.rpm"),
                &package("ibus-openbangla_3.0.0-fedora.rpm")
            ]
        );

        assert_eq!(
            select_packages(&packages, Dist::Tumbleweed),
            vec![
                &package("ibus-openbangla_3.0.0-opensuse-tumbleweed.rpm"),
                &package("fcitx-openbangla_3.0.0-opensuse-tumbleweed.rpm")
            ]
        );
    }
}
