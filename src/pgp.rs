use std::{fs, io::Write, sync::LazyLock};

use anyhow::Result;
use axum::{routing::get, Router};
use dotenvy::var;
use mongodb::Client;
// use pgp::{
//     cleartext::CleartextSignedMessage, ser::Serialize, types::SecretKeyTrait, ArmorOptions,
//     Deserializable, KeyType, Message, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey,
// };
// use rand::rngs::OsRng;

use sequoia_openpgp::{cert::prelude::*, parse::Parse, policy::StandardPolicy, serialize::{stream::{Armorer, Message, Signer}, SerializeInto}};

static CERT: LazyLock<Cert> = LazyLock::new(|| {
    let (cert, _) = CertBuilder::new()
        .add_userid("PackHub <sign@packhub.dev>")
        .add_signing_subkey()
        .generate()
        .unwrap();

    cert
});

static PASSPHRASE: LazyLock<String> = LazyLock::new(|| var("PACKHUB_SIGN_PASSPHRASE").unwrap());

// pub fn load_secret_key_from_file() -> Result<SignedSecretKey> {
//     let secret_key = std::fs::read("secret_key.asc")?;
//     let (signed_secret_key, _) = SignedSecretKey::from_armor_single(secret_key.as_slice())?;

//     Ok(signed_secret_key)
// }

// pub fn load_public_key_from_file() -> Result<SignedPublicKey> {
//     let public_key = std::fs::read("packhub.asc")?;
//     let (signed_public_key, _) = SignedPublicKey::from_armor_single(public_key.as_slice())?;

//     Ok(signed_public_key)
// }

pub fn clearsign_metadata(data: &str) -> Result<String> {
    let keypair = CERT
        .keys()
        .secret()
        .with_policy(&StandardPolicy::new(), None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .nth(0)
        .unwrap()
        .key()
        .clone()
        .into_keypair()
        .unwrap();

    let mut sink = vec![];
    let message = Message::new(&mut sink);
    let mut signer = Signer::new(message, keypair).unwrap()
        .cleartext()
        .build().unwrap();

    signer.write_all(data.as_bytes()).unwrap();
    signer.finalize().unwrap();

    Ok(String::from_utf8(sink).unwrap())
}

pub fn detached_sign_metadata(
    // file_name: &str,
    content: &str,
) -> Result<String> {
    let keypair = CERT
        .keys()
        .secret()
        .with_policy(&StandardPolicy::new(), None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .nth(0)
        .unwrap()
        .key()
        .clone()
        .into_keypair()
        .unwrap();

    let mut sink = vec![];
    let message = Armorer::new(Message::new(&mut sink)).build().unwrap();
    let mut signer = Signer::new(message, keypair).unwrap()
        .detached()
        .build().unwrap();

    signer.write_all(content.as_bytes()).unwrap();
    signer.finalize().unwrap();

    Ok(String::from_utf8(sink).unwrap())
}

// pub fn generate_secret_key() -> Result<SignedSecretKey> {
//     let mut key_params = SecretKeyParamsBuilder::default();
//     key_params
//         .key_type(KeyType::Rsa(2048))
//         .can_certify(false)
//         .can_sign(true)
//         .primary_user_id("PackHub Signing <sign@packhub.dev>".into());

//     let secret_key_params = key_params.build()?;
//     let secret_key = secret_key_params.generate(OsRng)?;
//     let passwd_fn = || PASSPHRASE.clone();
//     let signed_secret_key = secret_key.sign(OsRng, passwd_fn)?;

//     Ok(signed_secret_key)
// }

// pub fn generate_and_save_keys() -> Result<()> {
//     let secret_key = generate_secret_key()?;
//     let public_key = public_key_from_secret_key(&secret_key)?;

//     let secret_signed_key_armor = secret_key.to_armored_bytes(ArmorOptions::default())?;
//     fs::write("secret_key.asc", secret_signed_key_armor)?;

//     let public_signed_key_armor = public_key.to_armored_string(ArmorOptions::default())?;
//     fs::write("packhub.asc", public_signed_key_armor)?;

//     Ok(())
// }

// pub fn public_key_from_secret_key(secret_key: &SignedSecretKey) -> Result<SignedPublicKey> {
//     let public_key = secret_key.public_key();
//     Ok(public_key.sign(OsRng, &secret_key, || PASSPHRASE.clone())?)
// }

// fn dearmored_public_key() -> Vec<u8> {
//     let key = load_public_key_from_file().unwrap();
//     key.to_bytes().unwrap()
// }

pub fn keys() -> Router<Client> {
    Router::new()
        .route(
            "/packhub.asc",
            get(|| async { String::from_utf8(CERT.armored().to_vec().unwrap()).unwrap() }),
        )
        .route("/packhub.gpg", get(|| async { CERT.to_vec().unwrap()}))
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     const METADATA: &str = "Test Metadata";

//     #[test]
//     fn test_generate_key_and_message_verify() {
//         dotenvy::from_filename(".env.example").unwrap();

//         let secret_key = generate_secret_key().unwrap();
//         let public_key = public_key_from_secret_key(&secret_key).unwrap();

//         let clearsign_text = clearsign_metadata(METADATA, &secret_key).unwrap();

//         let (message, _) = CleartextSignedMessage::from_armor(clearsign_text.as_bytes()).unwrap();
//         assert!(message.verify(&public_key).is_ok());
//     }
// }
