use std::collections::HashMap;

use lenient_semver::parse;
use once_cell::sync::Lazy;
use regex::Regex;
use semver::VersionReq;

use crate::detect::Dist;

static APT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Debian APT.+\((.+)\)"#).unwrap());
static UBUNTU_VERSIONS: Lazy<HashMap<VersionReq, Dist>> = Lazy::new(|| {
    [
        matcher_ubuntu("=1.0.1", "14.04"),
        matcher_ubuntu(">=1.2.1, <=1.2.35", "16.04"),
        matcher_ubuntu(">=1.6.1, <=1.6.14", "18.04"),
        matcher_ubuntu(">=2.0.2, <=2.0.9", "20.04"),
        matcher_ubuntu(">=2.4.5, <=2.4.8", "22.04"),
        matcher_ubuntu("=2.5.3", "22.10"),
        matcher_ubuntu("=2.5.4", "23.04"),
    ]
    .into()
});

/// Returns the Ubuntu version matching to the `apt` version it comes with.
fn match_ubuntu_for_apt(ver: &str) -> Dist {
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

/// Creates a `VersionReq` and `Dist` tuple.
fn matcher_ubuntu(req: &str, ver: &str) -> (VersionReq, Dist) {
    (
        VersionReq::parse(req).unwrap(),
        Dist::Ubuntu(parse(ver).ok()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_platform() {
        assert_eq!(match_ubuntu_for_apt("1.0.1"), Dist::Ubuntu(parse("14.04").ok()));
        assert_eq!(match_ubuntu_for_apt("1.2.1"), Dist::Ubuntu(parse("16.04").ok()));
        assert_eq!(match_ubuntu_for_apt("1.2.10"), Dist::Ubuntu(parse("16.04").ok()));
        assert_eq!(match_ubuntu_for_apt("1.2.35"), Dist::Ubuntu(parse("16.04").ok()));
        assert_eq!(match_ubuntu_for_apt("1.6.1"), Dist::Ubuntu(parse("18.04").ok()));
        assert_eq!(match_ubuntu_for_apt("1.6.14"), Dist::Ubuntu(parse("18.04").ok()));
        assert_eq!(match_ubuntu_for_apt("2.0.2"), Dist::Ubuntu(parse("20.04").ok()));
        assert_eq!(match_ubuntu_for_apt("2.0.9"), Dist::Ubuntu(parse("20.04").ok()));
        assert_eq!(match_ubuntu_for_apt("2.4.5"), Dist::Ubuntu(parse("22.04").ok()));
        assert_eq!(match_ubuntu_for_apt("2.4.8"), Dist::Ubuntu(parse("22.04").ok()));
        assert_eq!(match_ubuntu_for_apt("2.5.3"), Dist::Ubuntu(parse("22.10").ok()));
        assert_eq!(match_ubuntu_for_apt("2.5.4"), Dist::Ubuntu(parse("23.04").ok()));
    }
}
