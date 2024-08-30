use leptos::{logging::log, *};
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
    
    let qr = move || {
        QrCode::with_version(jam_id(), Version::Normal(10), EcLevel::Q)
            .unwrap()
            .render()
            .quiet_zone(false)
            .min_dimensions(400, 400)
            .dark_color(svg::Color("#ffffff"))
            .light_color(svg::Color("#00000000"))
            .build()
    };
    let qr = Signal::derive(qr);

    let copy_to_clipboard = create_action(move |text: &String| {
        let text = text.clone();
        async move {
            save_to_clipboard(&text).await;
        }
    });

    view! {
        <div class="share">
            <div inner_html=qr></div>
            {jam_id}
            <button class="button" on:click=move |_| copy_to_clipboard.dispatch(jam_id())>
                "COPY"
            </button>
        </div>
    }
}
