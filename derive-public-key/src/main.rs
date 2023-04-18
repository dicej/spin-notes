use {
    clap::Parser,
    ed25519_dalek::{PublicKey, SecretKey},
};

/// Derives an ED25519 keypair from a salt and a password, outputting the public key to stdout
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Salt to use when deriving keypair
    salt: String,
}

fn main() {
    let args = Args::parse();

    let password = rpassword::prompt_password("password: ").unwrap();

    let mut key = [0u8; 32];
    scrypt::scrypt(
        password.as_bytes(),
        args.salt.as_bytes(),
        &scrypt::Params::default(),
        &mut key,
    )
    .unwrap();

    println!(
        "{}",
        hex::encode(PublicKey::from(&SecretKey::from_bytes(&key).unwrap()).to_bytes())
    );
}
