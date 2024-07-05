use leptos::{logging::warn, prelude::*, *};
use leptos_router::*;
use leptos_use::use_permission;
use web_sys::MediaDevices;

#[component]
pub fn CreateUserPage() -> impl IntoView {
    #[derive(PartialEq, Params)]
    struct JamId {
        id: String,
    }
    let jam_id = use_params::<JamId>();

    let jam_id = move || {
        jam_id.with(|jam_id| {
            jam_id
                .as_ref()
                .map(|jam_id| jam_id.id.clone())
                .unwrap_or_else(|_| {
                    let navigate = use_navigate();
                    navigate("/", NavigateOptions::default());
                    warn!("No jam id provided, redirecting to home page");
                    "".to_string()
                })
        })
    };

    if cfg!(any(target_arch = "wasm32", target_arch = "wasm64")) {
        use wasm_bindgen::prelude::*;
        use 
        let window=web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let camera=window.navigator().media_devices().unwrap().get_user_media().unwrap();
        let camera=JsFuture::from(camera).await.unwrap();
    }

    view! {
        <canvas id="canvas" style="display:none;"></canvas>
        <video id="video">"Video stream not available."</video>
        <img id="photo" alt="The screen capture will appear in this box."/>
        <button id="start-button">"Take photo"</button>
    }
}
