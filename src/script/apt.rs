use crate::utils::ReleaseChannel;
use askama::Template;

#[derive(Template)]
#[template(path = "apt-script.sh", escape = "none")]
pub(crate) struct AptScript<'a> {
    pub(crate) host: &'a str,
    pub(crate) distro: &'a str,
    pub(crate) owner: &'a str,
    pub(crate) repo: &'a str,
    pub(crate) channel: &'a ReleaseChannel,
}

pub(crate) fn generate_apt_script(
    distro: &str,
    owner: &str,
    repo: &str,
    channel: &ReleaseChannel,
) -> String {
    let host = dotenvy::var("PACKHUB_DOMAIN").unwrap();
    let script = AptScript {
        host: &host,
        distro,
        owner,
        repo,
        channel,
    };
    script.render().unwrap()
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_script_generation_apt() {
        let apt_script = generate_apt_script(
            "ubuntu",
            "OpenBangla",
            "OpenBangla-Keyboard",
            &ReleaseChannel::Stable,
        );
        assert_snapshot!(apt_script);

        let apt_script_unstable = generate_apt_script(
            "ubuntu",
            "OpenBangla",
            "OpenBangla-Keyboard",
            &ReleaseChannel::Unstable,
        );
        assert_snapshot!(apt_script_unstable);
    }
}
