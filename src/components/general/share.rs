use leptos::{logging::log, prelude::*, *};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode, Version};

async fn save_to_clipboard(text: &str) {
    log!("copying to clipboard: {}", text);
    let window = match web_sys::window() {
        Some(window) => window,
        None => {
            log!("failed to copy, window not available");
            return;
        }
    };
    let clip = window.navigator().clipboard();

    let promise = clip.write_text(text);
    if wasm_bindgen_futures::JsFuture::from(promise).await.is_err() {
        log!("failed to copy to clipboard");
    }
}

#[component]
pub fn Share(#[prop(into)] jam_id: Signal<String>) -> impl IntoView {
    let (base_url, set_base_url) = signal(String::new());

    if cfg!(target_arch = "wasm32") {
        match web_sys::window() {
            Some(window) => {
                let location = window.location();
                match location.origin() {
                    Ok(base_url) => {
                        set_base_url(base_url);
                    }
                    Err(e) => {
                        error!("error getting base url: {:?}", e);
                    }
                }
            }
            None => {
                error!("window not found");
            }
        }
    }

    let qr = move || {
        QrCode::with_version(
            jam_id.with(move |id| base_url.with(move |url| format!("{}/create-user/{}", url, id))),
            Version::Normal(10),
            EcLevel::Q,
        )
        .unwrap()
        .render()
        .quiet_zone(false)
        .min_dimensions(400, 400)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build()
    };
    let jam_id = move || jam_id().to_uppercase();
    let jam_id = Signal::derive(jam_id);

    view! {
        <div class="share">
            <div inner_html=qr></div>
            {jam_id}
            <button
                class="button"
                on:click=move |_| {
                    jam_id.with_untracked(|id| log!("{}", id));
                    spawn::spawn_local(async move {
                        save_to_clipboard(&jam_id.get_untracked()).await
                    });
                }
            >

                "COPY"
            </button>
        </div>
    }
}
