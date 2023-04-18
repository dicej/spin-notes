# spin-notes

This is a simple [Spin](https://github.com/fermyon/spin) app for storing and
retrieving end-to-end-encrypted notes.

Disclaimer: I don't recommend using this for anything serious.  It uses
password-derived keys for encryption and signing, meaning its security depends
on the amount of entropy in the password used.  Plus, this was thrown together
as a quick demo and may be broken in subtle or not-so-subtle ways.

## Building and Running

### Prerequisites

- [Rust](https://rustup.rs/)
- [Trunk](https://trunkrs.dev/#getting-started)
- [Spin](https://developer.fermyon.com/spin/quickstart)

If you're on a UNIX-style OS (or WSL2), this sequence of commands should install
everything you need:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --locked trunk wasm-bindgen-cli
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv ./spin /usr/local/bin/spin
```

Next, edit `spin.toml`, setting the `public_key` field to the output of the
following command, which will prompt you for the password you plan to use when
encrypting and decrypting your notes:

```
cargo run --release -p derive-public-key http://127.0.0.1:3000
```

Finally, build and run the app:

```
spin build --up
```

...and visit http://127.0.0.1:3000/ in your browser.

## Deploying

If you like, you can deploy the app to [Fermyon Cloud](https://cloud.fermyon.com):

```
spin deploy
```

That will generate a random URL (e.g. `https://notes-r9teztg7.fermyon.app`),
which we'll need to use to derive a new public key and redeploy.  Once again,
edit `spin.toml`, setting the `public_key` field to the output of the following
command, substituting the URL generated above for `$ORIGIN`:

```
cargo run --release -p derive-public-key $ORIGIN
```

And finally redeploy:

```
spin deploy
```
