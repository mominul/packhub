use std::{fs, sync::LazyLock};

use anyhow::Result;
use axum::{routing::get, Router};
use dotenvy::var;
use mongodb::Client;
use pgp::{
    cleartext::CleartextSignedMessage, ser::Serialize, types::SecretKeyTrait, ArmorOptions,
    Deserializable, KeyType, Message, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey,
};
use rand::rngs::OsRng;

static PASSPHRASE: LazyLock<String> = LazyLock::new(|| var("PACKHUB_SIGN_PASSPHRASE").unwrap());

pub fn load_secret_key_from_file() -> Result<SignedSecretKey> {
    let secret_key = std::fs::read("secret_key.asc")?;
    let (signed_secret_key, _) = SignedSecretKey::from_armor_single(secret_key.as_slice())?;

    Ok(signed_secret_key)
}

pub fn load_public_key_from_file() -> Result<SignedPublicKey> {
    let public_key = std::fs::read("packhub.asc")?;
    let (signed_public_key, _) = SignedPublicKey::from_armor_single(public_key.as_slice())?;

    Ok(signed_public_key)
}

pub fn clearsign_metadata(text: &str, secret_key: &SignedSecretKey) -> Result<String> {
    let clear_text = CleartextSignedMessage::sign(OsRng, text, secret_key, || PASSPHRASE.clone())?;

    Ok(clear_text.to_armored_string(ArmorOptions::default())?)
}

pub fn detached_sign_metadata(
    file_name: &str,
    content: &str,
    secret_key: &SignedSecretKey,
) -> Result<String> {
    let message = Message::new_literal(file_name, content);
    let message = message.sign(
        OsRng,
        &secret_key,
        || PASSPHRASE.clone(),
        secret_key.hash_alg(),
    )?;

    Ok(message
        .into_signature()
        .to_armored_string(ArmorOptions::default())?)
}

pub fn generate_secret_key() -> Result<SignedSecretKey> {
    let mut key_params = SecretKeyParamsBuilder::default();
    key_params
        .key_type(KeyType::Rsa(2048))
        .can_certify(false)
        .can_sign(true)
        .primary_user_id("Test <test@packhub.org>".into());

    let secret_key_params = key_params.build()?;
    let secret_key = secret_key_params.generate(OsRng)?;
    let passwd_fn = || PASSPHRASE.clone();
    let signed_secret_key = secret_key.sign(OsRng, passwd_fn)?;

    Ok(signed_secret_key)
}

pub fn generate_and_save_keys() -> Result<()> {
    let secret_key = generate_secret_key()?;
    let public_key = public_key_from_secret_key(&secret_key)?;

    let secret_signed_key_armor = secret_key.to_armored_bytes(ArmorOptions::default())?;
    fs::write("secret_key.asc", secret_signed_key_armor)?;

    let public_signed_key_armor = public_key.to_armored_string(ArmorOptions::default())?;
    fs::write("packhub.asc", public_signed_key_armor)?;

    Ok(())
}

pub fn public_key_from_secret_key(secret_key: &SignedSecretKey) -> Result<SignedPublicKey> {
    let public_key = secret_key.public_key();
    Ok(public_key.sign(OsRng, &secret_key, || PASSPHRASE.clone())?)
}

fn dearmored_public_key() -> Vec<u8> {
    let key = load_public_key_from_file().unwrap();
    key.to_bytes().unwrap()
}

pub fn keys() -> Router<Client> {
    Router::new()
        .route(
            "/packhub.asc",
            get(|| async { fs::read_to_string("packhub.asc").unwrap() }),
        )
        .route("/packhub.gpg", get(dearmored_public_key()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const METADATA: &str = "Test Metadata";

    #[test]
    fn test_generate_key_and_message_verify() {
        dotenvy::from_filename(".env.example").unwrap();

        let secret_key = generate_secret_key().unwrap();
        let public_key = public_key_from_secret_key(&secret_key).unwrap();

        let clearsign_text = clearsign_metadata(METADATA, &secret_key).unwrap();

        let (message, _) = CleartextSignedMessage::from_armor(clearsign_text.as_bytes()).unwrap();
        assert!(message.verify(&public_key).is_ok());
    }
}
