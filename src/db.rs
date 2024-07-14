use bson::doc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::package::{Data, Package};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct PackageMetadata {
    name: String,
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    created_at: DateTime<Utc>,
    metadata: String,
}

impl PackageMetadata {
    /// Create a new `PackageMetadata` from metadata of a `Package`.
    ///
    /// `None` is returned if the package metadata is not available.
    pub fn from_package(package: &Package) -> Option<Self> {
        let Data::Metadata(metadata) = package.data() else {
            return None;
        };

        Some(Self {
            name: package.file_name().to_owned(),
            created_at: package.creation_date().clone(),
            updated_at: package.updated_date().clone(),
            metadata,
        })
    }

    pub async fn retrieve_from(
        collection: &mongodb::Collection<PackageMetadata>,
        package: &Package,
    ) -> Option<Self> {
        collection
            .find_one(doc! { "name": package.file_name() })
            .await
            .unwrap()
    }

    pub fn data(self) -> String {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;

    use super::*;
    use mongodb::Client;
    use testcontainers_modules::{
        mongo::Mongo,
        testcontainers::{runners::AsyncRunner, ContainerAsync},
    };

    use crate::apt::DebianPackage;

    pub async fn setup_mongodb(container: &ContainerAsync<Mongo>) -> Client {
        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(27017).await.unwrap();

        mongodb::Client::with_uri_str(&format!("mongodb://{}:{}", host, port))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_retrieval() {
        let container = Mongo::default().start().await.unwrap();
        let client = setup_mongodb(&container).await;

        let package = Package::detect_package("OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb", "2.0.0".to_owned(), "https://github.com/OpenBangla/OpenBangla-Keyboard/releases/download/2.0.0/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb".to_owned(), DateTime::parse_from_rfc3339("2024-07-01T00:00:00Z").unwrap().into(), DateTime::parse_from_rfc3339("2024-07-09T00:00:00Z").unwrap().into()).unwrap();
        let data = read("data/OpenBangla-Keyboard_2.0.0-ubuntu20.04.deb").unwrap();
        package.set_package_data(data);

        let _ = DebianPackage::from_package(&package).unwrap();

        let db = client.database("github");
        let collection = db.collection::<PackageMetadata>("test");

        let metadata = PackageMetadata::from_package(&package).unwrap();

        collection.insert_one(&metadata).await.unwrap();

        let retrieved = PackageMetadata::retrieve_from(&collection, &package)
            .await
            .unwrap();

        assert_eq!(metadata, retrieved);
    }
}
