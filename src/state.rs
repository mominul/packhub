use std::sync::Arc;

use anyhow::Result;
use dotenvy::var;
use mongodb::Client;
use octocrab::{Octocrab, OctocrabBuilder};
use sequoia_openpgp::{Cert, crypto::Password, serialize::SerializeInto};

use crate::pgp::{
    clearsign_metadata, detached_sign_metadata, generate_and_save_keys, load_cert_from_file,
};

#[derive(Clone)]
pub struct AppState {
    state: Arc<InnerState>,
}

struct InnerState {
    db: Client,
    cert: Cert,
    github: Octocrab,
    passphrase: Password,
}

impl AppState {
    pub async fn initialize(generate_keys: bool) -> Self {
        let uri = format!(
            "mongodb://{}:{}@{}:27017",
            var("PACKHUB_DB_USER").unwrap(),
            var("PACKHUB_DB_PASSWORD").unwrap(),
            var("PACKHUB_DB_HOST").unwrap()
        );

        let client = Client::with_uri_str(uri).await.unwrap();
        let passphrase = var("PACKHUB_SIGN_PASSPHRASE").unwrap().into();

        let cert = if generate_keys {
            generate_and_save_keys(&passphrase).unwrap()
        } else {
            load_cert_from_file().unwrap()
        };

        let github = OctocrabBuilder::default()
            .personal_token(var("PACKHUB_GITHUB_PAT").unwrap())
            .build()
            .unwrap();

        Self {
            state: Arc::new(InnerState {
                db: client,
                cert,
                github,
                passphrase,
            }),
        }
    }

    /// Get a reference to the MongoDB client.
    pub fn db(&self) -> &Client {
        &self.state.db
    }

    /// Get a reference to the GitHub client.
    pub fn github(&self) -> &Octocrab {
        &self.state.github
    }

    pub fn clearsign_metadata(&self, data: &str) -> Result<String> {
        clearsign_metadata(data, &self.state.cert, &self.state.passphrase)
    }

    pub fn detached_sign_metadata(&self, data: &str) -> Result<String> {
        detached_sign_metadata(data, &self.state.cert, &self.state.passphrase)
    }

    pub fn armored_public_key(&self) -> Vec<u8> {
        self.state.cert.armored().to_vec().unwrap()
    }

    pub fn dearmored_public_key(&self) -> Vec<u8> {
        self.state.cert.to_vec().unwrap()
    }
}
