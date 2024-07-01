use crate::app::general::*;
use leptos::{logging::log, prelude::*, *};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode, Version};

#[cfg(web_sys_unstable_apis)]
async fn save_to_clipboard(text: &str) {
    let text = text.to_string();

    let window = match web_sys::window() {
        Some(window) => window,
        None => {
            log!("failed to copy, window not available");
            return;
        }
    };
    let nav = window.navigator().clipboard();
    match nav {
        Some(clip) => {
            let promise = clip.write_text(&text);
            if wasm_bindgen_futures::JsFuture::from(promise).await.is_err() {
                log!("failed to copy to clipboard");
            }
        }
        None => {
            log!("failed to copy, clipboard not available");
        }
    };
}

#[component]
pub fn Share(jam_id: JamId) -> impl IntoView {
    let url = format!("https://jam.leptos.dev/jam/{}", jam_id);
    let qr = QrCode::with_version(url, Version::Normal(1), EcLevel::Q).unwrap();
    let qr = qr
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build();

    view! {
        <div class="share">
            <svg viewBox="" inner_html=qr/>

        </div>
    }
}
