use anyhow::Result;

use crate::{distro::Distro, infra::InfraTest};

mod containers;
mod distro;
mod infra;

#[tokio::main]
async fn main() -> Result<()> {
    let mut infra = InfraTest::setup_infra().await?;

    infra.add_distro(Distro::new(
        "single-pkg",
        "ubuntu",
        "24.04",
        "check_apt.sh",
        Some(&[("DIST", "ubuntu")]),
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "ubuntu",
        "24.04",
        "check_apt_multiple.sh",
        Some(&[("DIST", "ubuntu")]),
    ));

    infra.add_distro(Distro::new(
        "single-pkg",
        "debian",
        "12",
        "check_apt.sh",
        Some(&[("DIST", "debian")]),
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "debian",
        "12",
        "check_apt_multiple.sh",
        Some(&[("DIST", "debian")]),
    ));

    infra.add_distro(Distro::new(
        "single-pkg",
        "fedora",
        "42",
        "check_dnf.sh",
        None,
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "fedora",
        "42",
        "check_dnf_multiple.sh",
        None,
    ));

    infra.add_distro(Distro::new(
        "multi-pkg",
        "opensuse/tumbleweed",
        "latest",
        "check_zypper_multiple.sh",
        None,
    ));

    // Pre-release support tests
    infra.add_distro(Distro::new(
        "pre-release-pkg",
        "ubuntu",
        "25.04",
        "check_apt_pre.sh",
        None,
    ));

    infra.add_distro(Distro::new(
        "pre-release-pkg",
        "fedora",
        "42",
        "check_dnf_pre.sh",
        None,
    ));

    // Test rpm v1 support
    infra.add_distro(Distro::new(
        "v1-rpm",
        "fedora",
        "42",
        "check_dnf_v1.sh",
        None,
    ));

    infra.run_distros().await?;

    Ok(())
}
