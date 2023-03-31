use chrono::{Utc, DateTime};

use crate::detect::Package;

struct Repository {
    project: String,
    updated: DateTime<Utc>,
    packages: Vec<Package>
}