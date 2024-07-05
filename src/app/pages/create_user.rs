use gloo::{ events::EventListener};
use leptos::{logging::{warn,log}, prelude::*, *};
use leptos_router::*;
use leptos_use::use_permission;
use tracing::event;
use web_sys::{DisplayMediaStreamConstraints, MediaDevices, MediaStream};

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
        let lol=create_action(|_: &()| lol());
        lol.dispatch(());
    }

    view! {
        <canvas id="canvas" style="display:none;"></canvas>
        <video id="video">"Video stream not available."</video>
        <img id="photo" alt="The screen capture will appear in this box."/>
        <button id="start-button">"Take photo"</button>
    }
}

async fn lol() {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    let width = 320;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    log!("got document and window");

    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    log!("got canvas");
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    log!("got context");

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
    log!("got camera promise");
    let camera = match JsFuture::from(camera).await{
        Ok(camera) => camera,
        Err(e) => {
            warn!("Error getting camera: {:?}", e);
            return;
        }
    
    };
    log!("got camera jsv");
    let camera = camera.dyn_into::<MediaStream>().unwrap();
    log!("got camera");

    let video = document.get_element_by_id("video").unwrap();
    let video = video
        .dyn_into::<web_sys::HtmlVideoElement>()
        .map_err(|_| ())
        .unwrap();
    log!("got video");
    video.set_src_object(Some(&camera));
    log!("set video src object");

    while video.video_width() == 0 {}

    let start_button = document.get_element_by_id("start-button").unwrap();
    let start_button = start_button
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    log!("got start button");
    let event_listener = EventListener::new(&start_button, "click", move |ev| {
        canvas.set_width(width);
        let height = video.video_height() / video.video_width() * width;
        canvas.set_height(height);
        context
            .draw_image_with_html_video_element_and_dw_and_dh(
                &video,
                0.0,
                0.0,
                width as f64,
                height as f64,
            )
            .unwrap();
        let data_url = canvas.to_data_url_with_type("image/png").unwrap();
        let photo = document.get_element_by_id("photo").unwrap();
        let photo = photo
            .dyn_into::<web_sys::HtmlImageElement>()
            .map_err(|_| ())
            .unwrap();
        photo.set_src(data_url.as_str());
        ev.prevent_default();
    });
    event_listener.forget();
}
