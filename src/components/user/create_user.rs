use crate::components::user::set_bg_img;
use gloo::storage::{LocalStorage, Storage};
use leptos::{logging::error, prelude::*, *};
use leptos_router::*;
use std::rc::Rc;
use web_sys::MediaStream;

#[server]
async fn create_user(
    jam_id: String,
    name: String,
    pfp_url: String,
) -> Result<String, ServerFnError> {
    use crate::general::notify;
    use crate::general::{functions::create_user as create_user_fn, types::AppState};
    let app_state = expect_context::<AppState>();
    let pool = &app_state.db.pool;
    match create_user_fn(&jam_id, &pfp_url, &name, pool).await {
        Ok(user_id) => {
            notify(user_id.1, vec![], &jam_id, pool).await?;
            Ok(user_id.0)
        }
        Err(e) => Err(ServerFnError::ServerError(e.into())),
    }
}

#[component]
pub fn CreateUser(jam_id: String) -> impl IntoView {
    let jam_id = Rc::new(jam_id);

    let (image_url, set_image_url) = create_signal(String::new());
    let (camera_request_state, set_camera_request_state) =
        create_signal(CameraRequestState::Asking);
    let video_id = "video";

    let camera = create_action(move |_: &()| async move {
        camera(set_image_url, set_camera_request_state, video_id).await
    });

    {
        let jam_id = Rc::clone(&jam_id);
        create_effect(move |_| {
            let jam_id: &str = &jam_id;
            let user_id: String = LocalStorage::get(jam_id).unwrap_or_default();
            if user_id.is_empty() {
                camera.dispatch(());
            } else if user_id == "kicked" {
                let navigate = use_navigate();
                navigate("/", NavigateOptions::default());
            } else {
                let navigate = use_navigate();
                navigate(&format!("/jam/{}", jam_id), NavigateOptions::default());
            }
        });
    }

    let take_picture = move || {
        camera.value().with(|response| {
            if let Some(response) = response {
                match response {
                    Ok(response) => (response.take_picture)(),
                    Err(e) => error!("Error taking picture: {:?}", e),
                }
            }
        })
    };
    let close_camera = move || {
        camera.value().with(|response| {
            if let Some(response) = response {
                match response {
                    Ok(response) => (response.close_camera)(),
                    Err(e) => error!("Error taking picture: {:?}", e),
                }
            }
        })
    };
    let (name, set_name) = create_signal(String::new());

    let create_user = create_action({
        let jam_id = jam_id.clone();
        move |_: &()| {
            let name = name.get();
            let pfp_url = image_url.get();
            let jam_id = (*jam_id).clone();
            async move { create_user(jam_id, name, pfp_url).await }
        }
    });

    create_effect(move |_| {
        if let Some(res) = create_user.value().get() {
            match res {
                Ok(id) => {
                    let jam_id: &str = &jam_id;
                    if let Err(e) = LocalStorage::set(jam_id, id) {
                        error!("Error setting user id in local storage: {:?}", e);
                    }
                    let navigate = use_navigate();
                    navigate(&format!("/jam/{}", jam_id), NavigateOptions::default());
                }
                Err(e) => error!("Error creating user: {:?}", e),
            }
        };
    });

    create_effect(move |_| {
        image_url.with(|url| {
            set_bg_img(url);
        })
    });

    view! {
        <div class="create-user">
            <div class="image-container">
                <video
                    style:display=move || {
                        if image_url.with(|url| !url.is_empty()) { "none" } else { "inline " }
                    }

                    playsinline="false"
                    disablepictureinpicture
                    disableremoteplayback

                    id=video_id
                >
                    "Video stream not available."
                </video>
                <img
                    class="photo"
                    style:display=move || {
                        if image_url.with(|url| url.is_empty()) { "none" } else { "inline " }
                    }

                    prop:src=image_url
                    alt="The screen capture will appear in this box."
                />
            </div>
            <input
                type="text"
                class="text-input"
                placeholder="Name"
                on:input=move |ev| set_name(event_target_value(&ev))
            />
            <div class="buttons">
                <button
                    class="capture-button"
                    style:display=move || {
                        if image_url.with(|url| !url.is_empty()) { "none" } else { "inline " }
                    }

                    on:click=move |_| {
                        if image_url.with(|url| url.is_empty()) {
                            take_picture();
                        }
                    }
                >

                    {move || {
                        if let CameraRequestState::Asking = camera_request_state.get() {
                            view! {
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="1em"
                                    height="1em"
                                    viewBox="0 0 24 24"
                                >
                                    <circle cx="18" cy="12" r="0">
                                        <animate
                                            attributeName="r"
                                            begin=".67"
                                            calcMode="spline"
                                            dur="1.5s"
                                            keySplines="0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8"
                                            repeatCount="indefinite"
                                            values="0;2;0;0"
                                        ></animate>
                                    </circle>
                                    <circle cx="12" cy="12" r="0">
                                        <animate
                                            attributeName="r"
                                            begin=".33"
                                            calcMode="spline"
                                            dur="1.5s"
                                            keySplines="0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8"
                                            repeatCount="indefinite"
                                            values="0;2;0;0"
                                        ></animate>
                                    </circle>
                                    <circle cx="6" cy="12" r="0">
                                        <animate
                                            attributeName="r"
                                            begin="0"
                                            calcMode="spline"
                                            dur="1.5s"
                                            keySplines="0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8"
                                            repeatCount="indefinite"
                                            values="0;2;0;0"
                                        ></animate>
                                    </circle>
                                </svg>
                            }
                                .into_view()
                        } else {
                            view! {
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    width="100"
                                    height="100"
                                    viewBox="0 0 100 100"
                                    fill="none"
                                >
                                    <path
                                        d="M100 50C100 77.6142 77.6142 100 50 100C22.3858 100 0 77.6142 0 50C0 22.3858 22.3858 0 50 0C77.6142 0 100 22.3858 100 50ZM7.5 50C7.5 73.4721 26.5279 92.5 50 92.5C73.4721 92.5 92.5 73.4721 92.5 50C92.5 26.5279 73.4721 7.5 50 7.5C26.5279 7.5 7.5 26.5279 7.5 50Z"
                                        fill="white"
                                    ></path>
                                </svg>
                            }
                                .into_view()
                        }
                    }}

                </button>
                <button
                    class="clear-button"
                    on:click=move |_| set_image_url(String::new())
                    style:display=move || {
                        if image_url.with(|url| url.is_empty()) { "none" } else { "inline " }
                    }
                >

                    <svg
                        viewBox="0 0 32 32"
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        stroke="#ffffff"
                    >
                        <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
                        <g
                            id="SVGRepo_tracerCarrier"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        ></g>
                        <g id="SVGRepo_iconCarrier">
                            <path
                                stroke="#ffffff"
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M8.5 23.5l15-15M23.5 23.5l-15-15"
                            ></path>
                        </g>
                    </svg>
                </button>
                <button
                    class="create-button"
                    on:click=move |_| {
                        create_user.dispatch(());
                        close_camera();
                    }

                    style:display=move || {
                        if image_url.with(|url| url.is_empty()) { "none" } else { "inline " }
                    }
                >

                    <svg
                        fill="#ffffff"
                        viewBox="0 0 32 32"
                        version="1.1"
                        xmlns="http://www.w3.org/2000/svg"
                        stroke="#ffffff"
                        stroke-width="0.00032"
                    >
                        <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
                        <g
                            id="SVGRepo_tracerCarrier"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        ></g>
                        <g id="SVGRepo_iconCarrier">
                            <title>checkmark2</title>
                            <path d="M28.998 8.531l-2.134-2.134c-0.394-0.393-1.030-0.393-1.423 0l-12.795 12.795-6.086-6.13c-0.393-0.393-1.029-0.393-1.423 0l-2.134 2.134c-0.393 0.394-0.393 1.030 0 1.423l8.924 8.984c0.393 0.393 1.030 0.393 1.423 0l15.648-15.649c0.393-0.392 0.393-1.030 0-1.423z"></path>
                        </g>
                    </svg>
                </button>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CameraRequestState {
    Denied,
    Asking,
    Granted,
}

struct CameraResponse {
    take_picture: Box<dyn Fn()>,
    close_camera: Box<dyn Fn()>,
}

async fn camera(
    image_url: WriteSignal<String>,
    camera_request_state: WriteSignal<CameraRequestState>,
    video_id: &str,
) -> Result<CameraResponse, String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = match web_sys::window() {
        Some(window) => window,
        None => {
            error!("Error getting window object");
            return Err("Error getting window object".to_string());
        }
    };
    let document = match window.document() {
        Some(document) => document,
        None => {
            error!("Error getting document object");
            return Err("Error getting document object".to_string());
        }
    };

    let canvas = match document.create_element("canvas") {
        Ok(canvas) => canvas,
        Err(e) => {
            error!("Error creating canvas element: {:?}", e);
            return Err(format!("Error creating canvas element: {:?}", e));
        }
    };

    let canvas = match canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
    {
        Ok(canvas) => canvas,
        Err(e) => {
            error!("Error mapping canvas element to canvas object: {:?}", e);
            return Err(format!(
                "Error mapping canvas element to canvas object: {:?}",
                e
            ));
        }
    };

    let context = match canvas.get_context("2d") {
        Ok(Some(context)) => context,
        Ok(None) => {
            error!("Error getting context: {:?}", "No context");
            return Err("No context".to_string());
        }
        Err(e) => {
            error!("Error getting context: {:?}", e);
            return Err(format!("Error getting context: {:?}", e));
        }
    };
    let context = match context.dyn_into::<web_sys::CanvasRenderingContext2d>() {
        Ok(context) => context,
        Err(e) => {
            error!("Error mapping context to object: {:?}", e);
            return Err(format!("Error mapping context to object: {:?}", e));
        }
    };

    let camera = match window.navigator().media_devices() {
        Ok(media_devices) => media_devices,
        Err(e) => {
            error!("Error getting media devices: {:?}", e);
            return Err(format!("Error getting media devices: {:?}", e));
        }
    };

    let camera_constraints = web_sys::MediaStreamConstraints::new();
    camera_constraints.set_video(&JsValue::from(true));
    camera_constraints.set_audio(&JsValue::from(false));

    let camera = match camera.get_user_media_with_constraints(&camera_constraints) {
        Ok(camera) => camera,
        Err(e) => {
            error!("Error getting camera promise: {:?}", e);
            return Err(format!("Error getting camera promise: {:?}", e));
        }
    };
    camera_request_state.set(CameraRequestState::Asking);
    let camera = match JsFuture::from(camera).await {
        Ok(camera) => camera,
        Err(e) => {
            camera_request_state.set(CameraRequestState::Denied);
            error!("Error resolving camera future: {:?}", e);
            return Err(format!("Error resolving camera future: {:?}", e));
        }
    };
    camera_request_state.set(CameraRequestState::Granted);
    let camera = match camera.dyn_into::<MediaStream>() {
        Ok(camera) => camera,
        Err(e) => {
            error!("Error mapping camera to object: {:?}", e);
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
            error!("Error playing video: {:?}", e);
            return Err(format!("Error playing video: {:?}", e));
        }
    };
    match JsFuture::from(promise).await {
        Ok(_) => (),
        Err(e) => {
            error!("Error resolving play promise: {:?}", e);
            return Err(format!("Error resolving play promise: {:?}", e));
        }
    };

    let capture = {
        let video = video.clone();
        Box::new(move || {
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
            let data_url = canvas.to_data_url_with_type("image/webp").unwrap();
            image_url(data_url.clone());
        })
    };

    let close_camera = Box::new(move || {
        video.pause().unwrap();
        video.set_src("");
        video.set_src_object(None);
        camera.get_tracks().iter().for_each(|track| {
            let track = track.dyn_into::<web_sys::MediaStreamTrack>().unwrap();
            track.stop();
        });
    });

    Ok(CameraResponse {
        take_picture: capture,
        close_camera,
    })
}
