use {
    anyhow::{anyhow, Result},
    ed25519_dalek::{PublicKey, Signature},
    http::{response::Builder, Method, StatusCode},
    spin_sdk::{
        config,
        http::{Request, Response},
        http_component,
        key_value::Store,
    },
    std::fs,
};

const NOTES_KEY: &str = "notes";
const SIGNATURE_HEADER: &str = "notes-signature";
const PUBLIC_KEY_CONFIG_KEY: &str = "public_key";

#[http_component]
fn handle_notes(req: Request) -> Result<Response> {
    let store = Store::open_default()?;

    Ok(match (req.method(), req.uri().path()) {
        (&Method::GET, "/notes") => response()
            .header("content-type", "text/plain")
            .body(Some(store.get(NOTES_KEY).unwrap_or_default().into()))?,

        (&Method::POST, "/notes") => match handle_post(
            &store,
            req.headers()
                .get(SIGNATURE_HEADER)
                .and_then(|v| v.to_str().ok()),
            req.body().as_deref(),
        ) {
            Ok(()) => response().body(None)?,
            Err(e) => {
                eprintln!("bad POST request: {e:?}");
                response().status(StatusCode::BAD_REQUEST).body(None)?
            }
        },

        (&Method::GET, path) => {
            if let Ok(body) = fs::read(path) {
                response()
                    .header(
                        "content-type",
                        mime_guess::from_path(path).first_raw().unwrap(),
                    )
                    .body(Some(body.into()))
            } else {
                response()
                    .header("content-type", "text/html")
                    .body(Some(fs::read("index.html")?.into()))
            }?
        }

        _ => response().status(StatusCode::BAD_REQUEST).body(None)?,
    })
}

fn response() -> Builder {
    http::Response::builder()
}

fn handle_post(store: &Store, signature: Option<&str>, body: Option<&[u8]>) -> Result<()> {
    let old_notes = store.get(NOTES_KEY).unwrap_or_default();
    let new_notes = body.unwrap_or(&[]);
    let signature =
        hex::decode(signature.ok_or_else(|| anyhow!("missing `notes-signature` header"))?)?;
    let public_key = hex::decode(config::get(PUBLIC_KEY_CONFIG_KEY)?)?;

    PublicKey::from_bytes(&public_key)?.verify_strict(
        &[old_notes.as_slice(), new_notes].concat(),
        &Signature::from_bytes(&signature)?,
    )?;

    store.set(NOTES_KEY, new_notes)?;

    Ok(())
}
