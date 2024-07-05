use gloo::events::EventListener;
use js_sys::Math::log;
use leptos::{
    logging::{log, warn},
    prelude::*,
    *,
};
use leptos_router::*;
use leptos_use::use_permission;
use tracing::event;
use web_sys::{DisplayMediaStreamConstraints, MediaDevices, MediaStream};

use crate::components::create;

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

    let (image_url, set_image_url) = create_signal(String::from("data:"));

    if cfg!(any(target_arch = "wasm32", target_arch = "wasm64")) {
        let lol = create_action(move |_: &()| camera(set_image_url));
        lol.dispatch(());
    }

    view! {
        <video id="video">"Video stream not available."</video>
        <img id="photo" src=image_url alt="The screen capture will appear in this box."/>
        <button id="start-button">"Take photo"</button>
    }
}

async fn camera(image_url: WriteSignal<String>) {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let canvas = document.create_element("canvas").unwrap();
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

    let camera = window
        .navigator()
        .media_devices()
        .unwrap()
        .get_user_media_with_constraints(
            web_sys::MediaStreamConstraints::new()
                .video(&JsValue::from(true))
                .audio(&JsValue::from(false)),
        )
        .unwrap();
    let camera = match JsFuture::from(camera).await {
        Ok(camera) => camera,
        Err(e) => {
            warn!("Error getting camera: {:?}", e);
            return;
        }
    };
    let camera = camera.dyn_into::<MediaStream>().unwrap();

    let video = document.get_element_by_id("video").unwrap();
    let video = video
        .dyn_into::<web_sys::HtmlVideoElement>()
        .map_err(|_| ())
        .unwrap();

    
    video.set_src_object(Some(&camera));
    let promise = video.play().unwrap();
    JsFuture::from(promise).await.unwrap();
   

    let start_button = document.get_element_by_id("start-button").unwrap();
    let start_button = start_button
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    let event_listener = EventListener::new(&start_button, "click", move |ev| {
        canvas.set_width(video.video_width());
        canvas.set_height(video.video_height());
        context
            .draw_image_with_html_video_element_and_dw_and_dh(
                &video,
                0.0,
                0.0,
                video.video_width() as f64,
                video.video_height() as f64,
            )
            .unwrap();
        let data_url = canvas.to_data_url_with_type("image/png").unwrap();
        image_url(data_url.clone());
        ev.prevent_default();
    });
    event_listener.forget();
}
