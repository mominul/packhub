//! This module contains the logic to select the best package for a given distribution.
//! It filters the packages based on the distribution and version, and returns the best match.

use std::collections::HashMap;

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
    // When there is no exact version match
    // but lower version match is available,
    // then we need to select the closest version.
    if !selective.is_empty() {
        // Group packages by name.
        let mut packages_by_name: HashMap<&str, Vec<&Package>> = HashMap::new();
        for package in selective.iter() {
            if let Some(name) = package.name() {
                packages_by_name.entry(name).or_default().push(*package);
            }
        }

        // Sort the packages by target distribution version and cut off greater versions.
        for (_, packages) in packages_by_name.iter_mut() {
            packages.sort_by(|a, b| b.distribution().unwrap().cmp(a.distribution().unwrap()));
            packages.retain(|i| dist >= *i.distribution().unwrap());
        }

        // Group by name and architecture
        let mut grouped_by_name_and_arch: HashMap<_, Vec<_>> = HashMap::new();
        for (name, packages) in packages_by_name.iter() {
            for package in packages.iter() {
                let arch = package.architecture();
                grouped_by_name_and_arch
                    .entry((name, arch))
                    .or_default()
                    .push(*package);
            }
        }

        // Take the first package from each group.
        // This will give us the packages that are closest to the target distribution.
        let merged = grouped_by_name_and_arch
            .into_values()
            .map(|v| v[0])
            .collect::<Vec<_>>();

        // If we have exact or relatively matched packages, then return them.
        if !merged.is_empty() {
            return merged;
        }

        // We have no exact or relative match, so return the selective packages.
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

    // We need to sort the packages to make the test deterministic.
    // Because `select_packages` sorts the packages by the distribution
    // which can change the order of the packages when multiple packages
    // are present. So we need to sort the packages by their file name.
    fn sort<'a>(mut v: Vec<&'a Package>) -> Vec<&'a Package> {
        v.sort();
        v
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
            select_packages(&packages, Dist::ubuntu("18.04")),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu18.04.deb")]
        );
        assert_eq!(
            select_packages(&packages, Dist::ubuntu("20.04")),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb")]
        );
        assert_eq!(
            select_packages(&packages, Dist::ubuntu("22.04")),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb")]
        );
    }

    #[test]
    fn test_package_selection_fedora() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            select_packages(&packages, Dist::fedora("38")),
            vec![&package("OpenBangla-Keyboard_2.0.0-fedora38.rpm")]
        );
    }

    #[test]
    fn test_package_selection_debian() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            select_packages(&packages, Dist::debian("11")),
            vec![&package("OpenBangla-Keyboard_2.0.0-debian11.deb")]
        );
    }

    #[test]
    fn test_multiple_package_selection() {
        let packages = multiple_packages();

        assert_eq!(
            sort(select_packages(&packages, Dist::ubuntu("22.04"))),
            vec![
                &package("fcitx-openbangla_3.0.0.deb"),
                &package("ibus-openbangla_3.0.0.deb")
            ]
        );

        assert_eq!(
            sort(select_packages(&packages, Dist::fedora("39"))),
            vec![
                &package("fcitx-openbangla_3.0.0-fedora.rpm"),
                &package("ibus-openbangla_3.0.0-fedora.rpm")
            ]
        );

        assert_eq!(
            sort(select_packages(&packages, Dist::Tumbleweed)),
            vec![
                &package("fcitx-openbangla_3.0.0-opensuse-tumbleweed.rpm"),
                &package("ibus-openbangla_3.0.0-opensuse-tumbleweed.rpm")
            ]
        );
    }

    #[test]
    fn test_package_selection_closest() {
        let packages: Vec<Package> = openbangla_keyboard_packages();

        assert_eq!(
            select_packages(&packages, Dist::fedora("41")),
            vec![&package("OpenBangla-Keyboard_2.0.0-fedora38.rpm")]
        );
        assert_eq!(
            select_packages(&packages, Dist::ubuntu("24.04")),
            vec![&package("OpenBangla-Keyboard_2.0.0-ubuntu22.04.deb")]
        );

        let packages = [
            package("flameshot-12.1.0-1-lp15.2.x86_64.rpm"),
            package("flameshot-12.1.0-1.debian-10.amd64.deb"),
            package("flameshot-12.1.0-1.debian-10.arm64.deb"),
            package("flameshot-12.1.0-1.debian-10.armhf.deb"),
            package("flameshot-12.1.0-1.debian-11.amd64.deb"),
            package("flameshot-12.1.0-1.debian-11.arm64.deb"),
            package("flameshot-12.1.0-1.debian-11.armhf.deb"),
            package("flameshot-12.1.0-1.fc35.x86_64.rpm"),
            package("flameshot-12.1.0-1.fc36.x86_64.rpm"),
            package("flameshot-12.1.0-1.ubuntu-20.04.amd64.deb"),
            package("flameshot-12.1.0-1.ubuntu-22.04.amd64.deb"),
        ];

        assert_eq!(
            select_packages(&packages, Dist::fedora("39")),
            vec![&package("flameshot-12.1.0-1.fc36.x86_64.rpm")]
        );

        assert_eq!(
            select_packages(&packages, Dist::ubuntu("21.04")),
            vec![&package("flameshot-12.1.0-1.ubuntu-20.04.amd64.deb")]
        );

        assert_eq!(
            select_packages(&packages, Dist::ubuntu("24.04")),
            vec![&package("flameshot-12.1.0-1.ubuntu-22.04.amd64.deb")]
        );

        assert_eq!(
            sort(select_packages(&packages, Dist::debian("12"))),
            vec![
                &package("flameshot-12.1.0-1.debian-11.amd64.deb"),
                &package("flameshot-12.1.0-1.debian-11.arm64.deb"),
                &package("flameshot-12.1.0-1.debian-11.armhf.deb")
            ]
        );
    }

    #[test]
    fn test_package_selection_without_dist() {
        let packages = [package("caprine_2.60.3_amd64.deb")];

        assert_eq!(
            select_packages(&packages, Dist::ubuntu("24.04")),
            vec![&package("caprine_2.60.3_amd64.deb")]
        );
    }
}
