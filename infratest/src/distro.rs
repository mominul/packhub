#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct Distro {
    pub(crate) name: String,
    pub(crate) image: String,
    pub(crate) tag: String,
    pub(crate) env: Option<Vec<(String, String)>>,
    pub(crate) script: String,
}

#[derive(Debug)]
pub(crate) struct DistroError {
    pub(crate) distro: Distro,
    pub(crate) code: i64,
}

impl std::fmt::Display for DistroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Distribution Container {}({}:{}) exited with error code: {}",
            self.distro.name, self.distro.image, self.distro.tag, self.code
        )
    }
}

impl std::error::Error for DistroError {}

impl Distro {
    pub(crate) fn new(
        name: &str,
        image: &str,
        tag: &str,
        script: &str,
        env: Option<&[(&str, &str)]>,
    ) -> Self {
        Distro {
            name: name.to_owned(),
            image: image.to_owned(),
            tag: tag.to_owned(),
            env: env.map(|env| {
                env.into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect::<Vec<(String, String)>>()
            }),
            script: script.to_owned(),
        }
    }
}
