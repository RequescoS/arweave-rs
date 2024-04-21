use crate::{
    crypto::hash::{deep_hash, ToItems},
    error::Error,
    transaction::Tx,
};
use data_encoding::BASE64URL;
use jsonwebkey::JsonWebKey;
use rand::thread_rng;
use rsa::pkcs8::FromPublicKey;
use rsa::{PaddingScheme, PublicKey, RsaPublicKey};
use sha2::Digest;

pub fn verify(pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), Error> {
    let jwt_str = format!(
        "{{\"kty\":\"RSA\",\"e\":\"AQAB\",\"n\":\"{}\"}}",
        BASE64URL.encode(pub_key)
    );
    let jwk: JsonWebKey = jwt_str.parse().unwrap();

    let pub_key = RsaPublicKey::from_public_key_der(jwk.key.to_der().as_slice()).unwrap();
    let mut hasher = sha2::Sha256::new();
    hasher.update(message);
    let hashed = &hasher.finalize();

    let rng = thread_rng();
    let padding = PaddingScheme::PSS {
        salt_rng: Box::new(rng),
        digest: Box::new(sha2::Sha256::new()),
        salt_len: None,
    };
    pub_key
        .verify(padding, hashed, signature)
        .map(|_| ())
        .map_err(|_| Error::InvalidSignature)
}

pub fn verify_transaction(transaction: &Tx) -> Result<(), Error> {
    if transaction.signature.is_empty() {
        return Err(Error::UnsignedTransaction);
    }

    let deep_hash_item = transaction.to_deep_hash_item()?;
    let message = deep_hash(deep_hash_item);
    let signature = &transaction.signature;

    let jwt_str = format!(
        "{{\"kty\":\"RSA\",\"e\":\"AQAB\",\"n\":\"{}\"}}",
        BASE64URL.encode(&transaction.owner.0)
    );
    let jwk: JsonWebKey = jwt_str.parse().unwrap();

    let pub_key = RsaPublicKey::from_public_key_der(jwk.key.to_der().as_slice()).unwrap();
    let mut hasher = sha2::Sha256::new();
    hasher.update(message);
    let hashed = &hasher.finalize();

    let rng = thread_rng();
    let padding = PaddingScheme::PSS {
        salt_rng: Box::new(rng),
        digest: Box::new(sha2::Sha256::new()),
        salt_len: None,
    };
    pub_key
        .verify(padding, hashed, &signature.0)
        .map(|_| ())
        .map_err(|_| Error::InvalidSignature)
}
