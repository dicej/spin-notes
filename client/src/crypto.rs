use {
    aes_gcm_siv::{aead::Aead, Aes256GcmSiv, KeyInit, Nonce},
    anyhow::{anyhow, Result},
    ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer},
};

const KEY_SIZE: usize = 32;
const SIGNATURE_SIZE: usize = 64;
const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 20;

fn derive_key(salt: &[u8], password: &str) -> [u8; KEY_SIZE] {
    let mut buffer = [0u8; KEY_SIZE];
    scrypt::scrypt(
        password.as_bytes(),
        salt,
        &scrypt::Params::default(),
        &mut buffer,
    )
    .unwrap();
    buffer
}

pub fn decrypt(ciphertext: &[u8], password: &str) -> Result<String> {
    let (nonce, body) = ciphertext.split_at(NONCE_SIZE);
    let (salt, body) = body.split_at(SALT_SIZE);

    Ok(String::from_utf8(
        Aes256GcmSiv::new_from_slice(&derive_key(salt, password))
            .unwrap()
            .decrypt(Nonce::from_slice(nonce), body)
            .map_err(|_| anyhow!("decryption failed"))?,
    )?)
}

pub fn encrypt(plaintext: &str, password: &str) -> Result<Vec<u8>> {
    let mut nonce = [0u8; NONCE_SIZE];
    getrandom::getrandom(&mut nonce).unwrap();

    let mut salt = [0u8; SALT_SIZE];
    getrandom::getrandom(&mut salt).unwrap();

    let mut buffer = Vec::with_capacity(NONCE_SIZE + SALT_SIZE + plaintext.len());

    buffer.extend(&nonce);
    buffer.extend(&salt);
    buffer.extend(
        Aes256GcmSiv::new_from_slice(&derive_key(&salt, password))
            .unwrap()
            .encrypt(Nonce::from_slice(&nonce), plaintext.as_bytes())
            .map_err(|_| anyhow!("encryption failed"))?,
    );

    Ok(buffer)
}

pub fn sign(message: &[u8], salt: &str, password: &str) -> [u8; SIGNATURE_SIZE] {
    let secret = SecretKey::from_bytes(&derive_key(salt.as_bytes(), password)).unwrap();
    let public = PublicKey::from(&secret);

    Keypair { secret, public }.sign(message).to_bytes()
}

pub fn public_key(salt: &str, password: &str) -> [u8; KEY_SIZE] {
    PublicKey::from(&SecretKey::from_bytes(&derive_key(salt.as_bytes(), password)).unwrap())
        .to_bytes()
}
