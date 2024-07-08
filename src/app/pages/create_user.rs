use gloo::events::EventListener;
use leptos::{
    logging::{log, warn},
    prelude::*,
    *,
};
use leptos_router::*;
use sqlx::pool;
use web_sys::MediaStream;

use crate::components::create;

#[server]
async fn create_user(
    jam_id: String,
    name: String,
    pfp_url: String,
) -> Result<String, ServerFnError> {
    use crate::app::general::{functions::create_user, types::AppState};
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    match create_user(&jam_id, &pfp_url, &name, pool).await {
        Ok(user_id) => Ok(user_id),
        Err(e) => Err(ServerFnError::ServerError(e.into())),
    }
}

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

    let video_id = "video";
    let take_picture = create_local_resource(
        || (),
        move |_| async move { camera(set_image_url, video_id).await },
    );

    match take_picture.get() {
        Some(f) => f(),
        None => warn!("Error taking picture: camera not initialized"),
    
    };

    view! {
        <video id=video_id>"Video stream not available."</video>
        <img id="photo" src=image_url alt="The screen capture will appear in this box."/>
        <button id="capture-button" on:click = move|_| {}>
            {move || if take_picture.loading().get() { "Loading..." } else { "Take picture" }}
        </button>
    }
}

async fn camera(image_url: WriteSignal<String>, take_picture:ReadSignal<()>, video_id: &str) -> Result<Box<dyn Fn()>, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = match web_sys::window() {
        Some(window) => window,
        None => {
            warn!("Error getting window object");
            return Err("Error getting window object".to_string());
        }
    };
    let document = match window.document() {
        Some(document) => document,
        None => {
            warn!("Error getting document object");
            return Err("Error getting document object".to_string());
        }
    };

    let canvas = match document.create_element("canvas") {
        Ok(canvas) => canvas,
        Err(e) => {
            warn!("Error creating canvas element: {:?}", e);
            return Err(format!("Error creating canvas element: {:?}", e));
        }
    };

    let canvas = match canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
    {
        Ok(canvas) => canvas,
        Err(e) => {
            warn!("Error mapping canvas element to canvas object: {:?}", e);
            return Err(format!(
                "Error mapping canvas element to canvas object: {:?}",
                e
            ));
        }
    };

    let context = match canvas.get_context("2d") {
        Ok(Some(context)) => context,
        Ok(None) => {
            warn!("Error getting context: {:?}", "No context");
            return Err("No context".to_string());
        }
        Err(e) => {
            warn!("Error getting context: {:?}", e);
            return Err(format!("Error getting context: {:?}", e));
        }
    };
    let context = match context.dyn_into::<web_sys::CanvasRenderingContext2d>() {
        Ok(context) => context,
        Err(e) => {
            warn!("Error mapping context to object: {:?}", e);
            return Err(format!("Error mapping context to object: {:?}", e));
        }
    };

    let camera = match window.navigator().media_devices() {
        Ok(media_devices) => media_devices,
        Err(e) => {
            warn!("Error getting media devices: {:?}", e);
            return Err(format!("Error getting media devices: {:?}", e));
        }
    };

    let camera = match camera.get_user_media_with_constraints(
        web_sys::MediaStreamConstraints::new()
            .video(&JsValue::from(true))
            .audio(&JsValue::from(false)),
    ) {
        Ok(camera) => camera,
        Err(e) => {
            warn!("Error getting camera promise: {:?}", e);
            return Err(format!("Error getting camera promise: {:?}", e));
        }
    };
    let camera = match JsFuture::from(camera).await {
        Ok(camera) => camera,
        Err(e) => {
            warn!("Error resolving camera future: {:?}", e);
            return Err(format!("Error resolving camera future: {:?}", e));
        }
    };
    let camera = match camera.dyn_into::<MediaStream>() {
        Ok(camera) => camera,
        Err(e) => {
            warn!("Error mapping camera to object: {:?}", e);
            return Err(format!("Error mapping camera to object: {:?}", e));
        }
    };

    let video = document.get_element_by_id(video_id).unwrap();
    let video = video
        .dyn_into::<web_sys::HtmlVideoElement>()
        .map_err(|_| ())
        .unwrap();

    video.set_src_object(Some(&camera));
    let promise = match video.play() {
        Ok(promise) => promise,
        Err(e) => {
            warn!("Error playing video: {:?}", e);
            return Err(format!("Error playing video: {:?}", e));
        }
    };
    match JsFuture::from(promise).await {
        Ok(_) => (),
        Err(e) => {
            warn!("Error resolving play promise: {:?}", e);
            return Err(format!("Error resolving play promise: {:?}", e));
        }
    };

    let f = move || {
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
    };
    Ok(Box::new(f))
}
