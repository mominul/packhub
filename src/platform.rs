use std::collections::HashMap;

use lenient_semver::parse;
use once_cell::sync::Lazy;
use regex::Regex;
use semver::VersionReq;

use crate::package::Dist;

static APT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Debian APT.+\((.+)\)"#).unwrap());
static FEDORA: Lazy<Regex> = Lazy::new(|| Regex::new(r#"libdnf \(Fedora Linux (\d+);"#).unwrap());

static UBUNTU_VERSIONS: Lazy<HashMap<VersionReq, Dist>> = Lazy::new(|| {
    [
        matcher_ubuntu("=1.0.1", "14.04"),
        matcher_ubuntu(">=1.2.1, <=1.2.35", "16.04"),
        matcher_ubuntu(">=1.6.1, <=1.6.14", "18.04"),
        matcher_ubuntu(">=2.0.2, <=2.0.10", "20.04"),
        matcher_ubuntu(">=2.4.5, <=2.4.10", "22.04"),
        matcher_ubuntu("=2.5.3", "22.10"),
        matcher_ubuntu(">=2.5.4, <=2.6.0", "23.04"),
        matcher_ubuntu("=2.7.3", "23.10"),
    ]
    .into()
});
static DEBIAN_VERSIONS: Lazy<HashMap<VersionReq, Dist>> = Lazy::new(|| {
    [
        matcher_debian("=1.0.9", "8"),
        matcher_debian(">=1.4.10, <=1.4.11", "9"),
        matcher_debian("=1.8.2+3", "10"),
        matcher_debian("=2.2.4", "11"),
        matcher_debian(">=2.5.4, <=2.6.1", "12"),
        matcher_debian("=2.7.6", "13"),
    ]
    .into()
});

/// Returns the Ubuntu version matching to the `apt` version it comes with.
pub(crate) fn match_ubuntu_for_apt(ver: &str) -> Dist {
    // TODO: handle cases like `1.0.1ubuntu2` which parses as `1.2.0-10ubuntu1`.
    let mut dist = Dist::Ubuntu(None);

    for (matcher, dst) in UBUNTU_VERSIONS.iter() {
        if matcher.matches(&parse(ver).unwrap()) {
            dist = dst.clone();
            break;
        }
    }

    dist
}

/// Returns the Debian version matching to the `apt` version it comes with.
fn match_debian_for_apt(ver: &str) -> Dist {
    let mut dist = Dist::Debian(None);

    for (matcher, dst) in DEBIAN_VERSIONS.iter() {
        if matcher.matches(&parse(ver).unwrap()) {
            dist = dst.clone();
            break;
        }
    }

    dist
}

/// Creates a `VersionReq` and `Dist` tuple for Ubuntu.
fn matcher_ubuntu(req: &str, ver: &str) -> (VersionReq, Dist) {
    (
        VersionReq::parse(req).unwrap(),
        Dist::Ubuntu(parse(ver).ok()),
    )
}

/// Creates a `VersionReq` and `Dist` tuple for Debian.
fn matcher_debian(req: &str, ver: &str) -> (VersionReq, Dist) {
    (
        VersionReq::parse(req).unwrap(),
        Dist::Debian(parse(ver).ok()),
    )
}

/// Retrieve the `apt` version from the user-agent string.
pub fn get_apt_version(agent: &str) -> &str {
    APT.captures(agent).unwrap().get(1).unwrap().as_str()
}

/// Retrieve the `apt` version from the user-agent string.
pub fn get_fedora_version(agent: &str) -> Option<&str> {
    Some(FEDORA.captures(agent)?.get(1)?.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_platform() {
        assert_eq!(
            match_ubuntu_for_apt("1.0.1"),
            Dist::Ubuntu(parse("14.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("1.2.1"),
            Dist::Ubuntu(parse("16.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("1.2.10"),
            Dist::Ubuntu(parse("16.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("1.2.35"),
            Dist::Ubuntu(parse("16.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("1.6.1"),
            Dist::Ubuntu(parse("18.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("1.6.14"),
            Dist::Ubuntu(parse("18.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.0.2"),
            Dist::Ubuntu(parse("20.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.0.9"),
            Dist::Ubuntu(parse("20.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.4.5"),
            Dist::Ubuntu(parse("22.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.4.8"),
            Dist::Ubuntu(parse("22.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.4.10"),
            Dist::Ubuntu(parse("22.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.5.3"),
            Dist::Ubuntu(parse("22.10").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.5.4"),
            Dist::Ubuntu(parse("23.04").ok())
        );
        assert_eq!(
            match_ubuntu_for_apt("2.6.0"),
            Dist::Ubuntu(parse("23.04").ok())
        );
        // FIXME: assert_eq!(match_ubuntu_for_apt("2.6.0ubuntu0.1"), Dist::Ubuntu(parse("23.04").ok()));
        assert_eq!(
            match_ubuntu_for_apt("2.7.3"),
            Dist::Ubuntu(parse("23.10").ok())
        );

        assert_eq!(match_debian_for_apt("1.0.9"), Dist::Debian(parse("8").ok()));
        assert_eq!(
            match_debian_for_apt("1.4.10"),
            Dist::Debian(parse("9").ok())
        );
        assert_eq!(
            match_debian_for_apt("1.4.11"),
            Dist::Debian(parse("9").ok())
        );
        assert_eq!(
            match_debian_for_apt("1.8.2.3"),
            Dist::Debian(parse("10").ok())
        );
        assert_eq!(
            match_debian_for_apt("2.2.4"),
            Dist::Debian(parse("11").ok())
        );
        assert_eq!(
            match_debian_for_apt("2.5.4"),
            Dist::Debian(parse("12").ok())
        );
        assert_eq!(
            match_debian_for_apt("2.6.1"),
            Dist::Debian(parse("12").ok())
        );
        assert_eq!(
            match_debian_for_apt("2.7.6"),
            Dist::Debian(parse("13").ok())
        );
    }

    #[test]
    fn test_apt_version() {
        assert_eq!(get_apt_version("Debian APT-HTTP/1.3 (2.5.3)"), "2.5.3");
    }

    #[test]
    fn test_fedora_version() {
        assert_eq!(get_fedora_version("libdnf (Fedora Linux 38; container; Linux.x86_64)"), Some("38"));
        assert_eq!(get_fedora_version("libdnf (Fedora Linux 39; container; Linux.x86_64)"), Some("39"));
    }
}
