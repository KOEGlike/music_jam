use leptos::{logging::log, *};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode, Version};

#[cfg(web_sys_unstable_apis)]
async fn save_to_clipboard(text: &str) {
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
            let promise = clip.write_text(text);
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
pub fn Share(
    #[prop(into)]
    jam_id: String
) -> impl IntoView {
    
    
    let url = format!("https://jam.leptos.dev/jam/{}", jam_id);
    let qr = QrCode::with_version(url.clone(), Version::Normal(15), EcLevel::Q).unwrap();
    let qr = qr
        .render()
        .min_dimensions(400, 400)
        .dark_color(svg::Color("#ffffff"))
        .light_color(svg::Color("#00000000"))
        .build();

    let copy_to_clipboard = create_action(move |text:&String| {
        let text = text.clone();
        async move {
            save_to_clipboard(&text).await;
        }
    });

    view! {
        <div class="share">
            <svg viewBox="" inner_html=qr></svg>
            {jam_id}
            <button on:click=move |_| copy_to_clipboard.dispatch(url.clone())>"COPY"</button>
        </div>
    }
}
