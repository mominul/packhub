use std::{fs, io::Write};

use anyhow::Result;
use axum::{extract::State, routing::get, Router};
use sequoia_openpgp::{
    cert::prelude::*,
    parse::Parse,
    policy::StandardPolicy,
    serialize::{
        stream::{Armorer, Message, Signer},
        SerializeInto,
    },
};

use crate::state::AppState;

fn generate_keys() -> Result<Cert> {
    let (cert, _) = CertBuilder::new()
        .add_userid("PackHub <sign@packhub.dev>")
        .add_signing_subkey()
        .generate()?;

    Ok(cert)
}

pub fn generate_and_save_keys() -> Result<Cert> {
    let cert = generate_keys()?;

    let key = cert.as_tsk().to_vec()?;

    fs::write("key.gpg", key)?;

    Ok(cert)
}

pub fn load_cert_from_file() -> Result<Cert> {
    let key = fs::read("key.gpg")?;
    let cert = Cert::from_bytes(&key)?;

    Ok(cert)
}

pub fn clearsign_metadata(data: &str, cert: &Cert) -> Result<String> {
    let keypair = cert
        .keys()
        .secret()
        .with_policy(&StandardPolicy::new(), None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .unwrap()
        .key()
        .clone()
        .into_keypair()
        .unwrap();

    let mut sink = vec![];
    let message = Message::new(&mut sink);
    let mut signer = Signer::new(message, keypair)?.cleartext().build()?;

    signer.write_all(data.as_bytes())?;
    signer.finalize()?;

    Ok(String::from_utf8(sink)?)
}

pub fn detached_sign_metadata(content: &str, cert: &Cert) -> Result<String> {
    let keypair = cert
        .keys()
        .secret()
        .with_policy(&StandardPolicy::new(), None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .unwrap()
        .key()
        .clone()
        .into_keypair()
        .unwrap();

    let mut sink = vec![];
    let message = Armorer::new(Message::new(&mut sink)).build()?;
    let mut signer = Signer::new(message, keypair)?.detached().build()?;

    signer.write_all(content.as_bytes())?;
    signer.finalize()?;

    Ok(String::from_utf8(sink)?)
}

/////////////////////////////////////// Axum handlers /////////////////////////////////////////////////

async fn armored_public_key_handler(State(state): State<AppState>) -> Vec<u8> {
    state.armored_public_key()
}

async fn dearmored_public_key_handler(State(state): State<AppState>) -> Vec<u8> {
    state.dearmored_public_key()
}

pub fn keys() -> Router<AppState> {
    Router::new()
        .route("/packhub.asc", get(armored_public_key_handler))
        .route("/packhub.gpg", get(dearmored_public_key_handler))
}
