use askama::Template;
use crate::utils::ReleaseChannel;
use crate::utils::AppVersion;

#[derive(Template)]
#[template(path = "rpm-script.sh", escape = "none")]
pub(crate) struct RPMScript<'a> {
    pub(crate) host: &'a str,
    pub(crate) owner: &'a str,
    pub(crate) repo: &'a str,
    pub(crate) mgr: &'a str,
    pub(crate) ver: &'a AppVersion,
    pub(crate) channel: &'a ReleaseChannel,
}

impl RPMScript<'_> {
    pub(crate) fn base_url(&self) -> String {
        match self.ver {
            AppVersion::V1 => format!(
                "{}/{}/rpm/github/{}/{}",
                self.host, "v1", self.owner, self.repo
            ),
            AppVersion::V2 => format!(
                "{}/{}/rpm/github/{}/{}/{}",
                self.host, "v2", self.owner, self.repo, self.channel
            ),
        }
    }

    pub(crate) fn repo_name(&self) -> String {
        match self.channel {
            ReleaseChannel::Stable => self.repo.to_string(),
            ReleaseChannel::Unstable => format!("{}-unstable", self.repo),
        }
    }

    pub(crate) fn name(&self) -> String {
        match self.channel {
            ReleaseChannel::Stable => self.repo.to_string(),
            ReleaseChannel::Unstable => format!("{} (unstable)", self.repo),
        }
    }
}

/// Generate RPM installation script
///
/// When `ver` is V1, the `channel` is ignored.
pub(crate) fn generate_rpm_script(
    owner: &str,
    repo: &str,
    mgr: &str,
    ver: &AppVersion,
    channel: &ReleaseChannel,
) -> String {
    let host = dotenvy::var("PACKHUB_DOMAIN").unwrap();
    let script = RPMScript {
        host: &host,
        owner,
        repo,
        mgr,
        ver,
        channel,
    };
    script.render().unwrap()
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_script_generation_rpm_v1() {
        let yum = generate_rpm_script(
            "OpenBangla",
            "OpenBangla-Keyboard",
            "yum.repos.d",
            &AppVersion::V1,
            &ReleaseChannel::Stable,
        );
        assert_snapshot!(yum);

        let zypp = generate_rpm_script(
            "OpenBangla",
            "OpenBangla-Keyboard",
            "zypp/repos.d",
            &AppVersion::V1,
            &ReleaseChannel::Stable,
        );
        assert_snapshot!(zypp);
    }

    #[test]
    fn test_script_generation_rpm_v2() {
        let yum = generate_rpm_script(
            "OpenBangla",
            "OpenBangla-Keyboard",
            "yum.repos.d",
            &AppVersion::V2,
            &ReleaseChannel::Stable,
        );
        assert_snapshot!(yum);

        let zypp = generate_rpm_script(
            "OpenBangla",
            "OpenBangla-Keyboard",
            "zypp/repos.d",
            &AppVersion::V2,
            &ReleaseChannel::Stable,
        );
        assert_snapshot!(zypp);

        let yum_unstable = generate_rpm_script(
            "OpenBangla",
            "OpenBangla-Keyboard",
            "yum.repos.d",
            &AppVersion::V2,
            &ReleaseChannel::Unstable,
        );
        assert_snapshot!(yum_unstable);

        let zypp_unstable = generate_rpm_script(
            "OpenBangla",
            "OpenBangla-Keyboard",
            "zypp/repos.d",
            &AppVersion::V2,
            &ReleaseChannel::Unstable,
        );
        assert_snapshot!(zypp_unstable);
    }
}
