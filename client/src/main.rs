use {
    anyhow::{anyhow, Error, Result},
    futures::future::TryFutureExt,
    js_sys::Uint8Array,
    leptos::{
        html::{Input, Textarea},
        IntoView, Scope, SignalSet,
    },
    reqwasm::http::{Request, Response},
};

mod crypto;

const SIGNATURE_HEADER: &str = "notes-signature";

fn main() {
    console_error_panic_hook::set_once();

    _ = console_log::init_with_level(log::Level::Debug);

    leptos::mount_to_body(notes);
}

fn notes(cx: Scope) -> impl IntoView {
    let origin = leptos::store_value(cx, web_sys::window().unwrap().location().origin().unwrap());
    let old_ciphertext = leptos::store_value(cx, Vec::new());
    let password_ref = leptos::create_node_ref::<Input>(cx);
    let notes_ref = leptos::create_node_ref::<Textarea>(cx);
    let notes = leptos::create_rw_signal(cx, String::new());
    let error = leptos::create_rw_signal(cx, String::new());
    let success = leptos::create_rw_signal(cx, String::new());

    let load = move |_| {
        error.set(String::new());
        success.set(String::new());

        wasm_bindgen_futures::spawn_local(
            async move {
                let password = password_ref.get().unwrap().value();
                let origin = origin.get_value();
                let ciphertext =
                    check_response(Request::get(&format!("{origin}/notes")).send().await?)?
                        .binary()
                        .await?;

                notes.set(if ciphertext.is_empty() {
                    String::new()
                } else {
                    crypto::decrypt(&ciphertext, &password)?
                });

                old_ciphertext.set_value(ciphertext);

                success.set("load was successful!".to_owned());

                Ok::<_, Error>(())
            }
            .unwrap_or_else(move |e| {
                let message = format!("error retrieving or decrypting notes: {e:?}");
                log::error!("{message}");
                error.set(message);
            }),
        )
    };

    let save = move |_| {
        error.set(String::new());
        success.set(String::new());

        wasm_bindgen_futures::spawn_local(
            async move {
                let password = password_ref.get().unwrap().value();
                let origin = origin.get_value();
                let old = old_ciphertext.get_value();
                let new = crypto::encrypt(&notes_ref.get().unwrap().value(), &password)?;
                let signature = hex::encode(crypto::sign(
                    &[old.clone(), new.clone()].concat(),
                    &origin,
                    &password,
                ));

                check_response(
                    Request::post(&format!("{origin}/notes"))
                        .header(SIGNATURE_HEADER, &signature)
                        .body(Uint8Array::from(new.as_slice()))
                        .send()
                        .await?,
                )?;

                old_ciphertext.set_value(new);

                success.set("save was successful!".to_owned());

                Ok::<_, Error>(())
            }
            .unwrap_or_else(move |e| {
                let public_key = hex::encode(crypto::public_key(
                    &origin.get_value(),
                    &password_ref.get().unwrap().value(),
                ));
                let message =
                    format!("error encrypting or sending notes: {e:?} (public key: {public_key})");
                log::error!("{message}");
                error.set(message);
            }),
        )
    };

    leptos::view! { cx,
        <div>
            <div>
                <label for="password">"password "</label>
                <input type="password" id="password" autofocus node_ref=password_ref/>
                <button on:click=load>"load"</button>
            </div>
            <div>
                <label for="notes">"notes "</label>
                <textarea id="notes" rows=10 cols=80 node_ref=notes_ref>
                    {notes}
                </textarea>
                <button on:click=save>"Save"</button>
            </div>
            <div style="color: #f54">
                {error}
            </div>
            <div style="color: #091">
                {success}
            </div>
        </div>
    }
}

fn check_response(response: Response) -> Result<Response> {
    if response.ok() {
        Ok(response)
    } else {
        Err(anyhow!("unexpected response status: {}", response.status()))
    }
}
