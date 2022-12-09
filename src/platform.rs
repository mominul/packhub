use once_cell::sync::Lazy;
use regex::Regex;

static APT: Lazy<Regex> = Lazy::new(|| Regex::new(r#"Debian APT.+\((.+)\)"#).unwrap());

struct Platform {
    //
}

impl Platform {
    pub(crate) fn detect_platform(agent: &str) -> Self {
        
        todo!()
    }
}