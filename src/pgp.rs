use std::{fs, io::Write};

use anyhow::Result;
use axum::{Router, extract::State, routing::get};
use sequoia_openpgp::{
    armor::Kind,
    cert::prelude::*,
    crypto::Password,
    parse::Parse,
    policy::StandardPolicy,
    serialize::{
        SerializeInto,
        stream::{Armorer, Message, Signer},
    },
};

use crate::state::AppState;

fn generate_keys(passphrase: &Password) -> Result<Cert> {
    let (cert, _) = CertBuilder::new()
        .add_userid("PackHub <sign@packhub.dev>")
        .set_password(Some(passphrase.clone()))
        .add_signing_subkey()
        .generate()?;

    Ok(cert)
}

pub fn generate_and_save_keys(passphrase: &Password) -> Result<Cert> {
    let cert = generate_keys(passphrase)?;

    let key = cert.as_tsk().to_vec()?;

    fs::write("key.gpg", key)?;

    Ok(cert)
}

pub fn load_cert_from_file() -> Result<Cert> {
    let key = fs::read("key.gpg")?;
    let cert = Cert::from_bytes(&key)?;

    Ok(cert)
}

pub fn clearsign_metadata(data: &str, cert: &Cert, passphrase: &Password) -> Result<Vec<u8>> {
    let binding = StandardPolicy::new();
    let key = cert
        .keys()
        .secret()
        .with_policy(&binding, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .unwrap()
        .key()
        .clone();

    let decrypted_key = key.decrypt_secret(passphrase)?;
    let keypair = decrypted_key.into_keypair()?;

    let mut sink = vec![];
    let message = Message::new(&mut sink);
    let mut signer = Signer::new(message, keypair)?.cleartext().build()?;

    signer.write_all(data.as_bytes())?;
    signer.finalize()?;

    Ok(sink)
}

pub fn detached_sign_metadata(
    content: &str,
    cert: &Cert,
    passphrase: &Password,
) -> Result<Vec<u8>> {
    let binding = StandardPolicy::new();
    let key = cert
        .keys()
        .secret()
        .with_policy(&binding, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .unwrap()
        .key()
        .clone();

    let decrypted_key = key.decrypt_secret(passphrase)?;
    let keypair = decrypted_key.into_keypair()?;

    let mut sink = vec![];
    let message = Armorer::new(Message::new(&mut sink))
        .kind(Kind::Signature)
        .build()?;
    let mut signer = Signer::new(message, keypair)?.detached().build()?;

    signer.write_all(content.as_bytes())?;
    signer.finalize()?;

    Ok(sink)
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

#[cfg(test)]
mod tests {
    use std::io::Read;

    use anyhow::Result;
    use sequoia_openpgp::{
        cert::prelude::*,
        parse::Parse,
        parse::stream::{MessageLayer, MessageStructure, VerificationHelper, VerifierBuilder},
        policy::StandardPolicy,
    };

    use super::*;

    struct VerificationHelperImpl {
        public_key: Cert,
    }

    impl VerificationHelper for VerificationHelperImpl {
        fn get_certs(
            &mut self,
            _ids: &[sequoia_openpgp::KeyHandle],
        ) -> sequoia_openpgp::Result<Vec<Cert>> {
            Ok(vec![self.public_key.clone()])
        }

        fn check(&mut self, structure: MessageStructure<'_>) -> sequoia_openpgp::Result<()> {
            for layer in structure.into_iter() {
                match layer {
                    MessageLayer::SignatureGroup { ref results } => {
                        // Simply check if all signatures are valid
                        if !results.iter().any(|r| r.is_ok()) {
                            return Err(anyhow::anyhow!("No valid signature"));
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_pgp_sign_and_verify() -> Result<()> {
        let passphrase = "secure-passphrase".into();

        // Generate new PGP key pair
        let cert = generate_keys(&passphrase)?;
        let message = "Test message to be signed";

        // Sign the message using cleartext signing
        let signed_message = clearsign_metadata(message, &cert, &passphrase)?;

        // Set up verification
        let helper = VerificationHelperImpl {
            public_key: cert.clone(),
        };
        let policy = StandardPolicy::new();

        // Create verifier with our helper
        let mut verifier =
            VerifierBuilder::from_bytes(&signed_message)?.with_policy(&policy, None, helper)?;

        // Read the verified content
        let mut verified_content = Vec::new();
        verifier.read_to_end(&mut verified_content)?;
        let verified_text = String::from_utf8(verified_content)?;

        // Verify the content matches original message
        assert_eq!(verified_text.trim(), message.trim());

        Ok(())
    }
}
