use crate::components::general::set_bg_img;
use gloo::{
    events::EventListener,
    storage::{LocalStorage, Storage},
};
use leptos::{
    either::Either,
    logging::{error, log},
    prelude::*,
    *,
};
use leptos_router::{hooks::use_navigate, *};
use std::{rc::Rc, time::Duration};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{FileReader, HtmlInputElement, MediaStream};

#[server]
async fn create_user(
    jam_id: String,
    name: String,
    pfp_url: String,
) -> Result<String, ServerFnError> {
    use crate::model::notify;
    use crate::model::{functions::create_user as create_user_fn, types::AppState};

    let app_state = expect_context::<AppState>();
    let mut transaction = app_state.db.pool.begin().await?;
    let res = match create_user_fn(
        &jam_id,
        &pfp_url,
        &name,
        &mut *transaction,
        &app_state.leptos_options.site_root,
    )
    .await
    {
        Ok(user_id) => {
            notify(user_id.1, vec![], &jam_id, &mut transaction).await?;
            Ok(user_id.0)
        }
        Err(e) => Err(ServerFnError::ServerError(e.into())),
    };
    transaction.commit().await?;
    res
}

#[component]
pub fn CreateUser(jam_id: String) -> impl IntoView {
    let jam_id = Rc::new(jam_id);

    let (image_url, set_image_url) = signal(String::new());
    let (camera_request_state, set_camera_request_state) = signal(CameraRequestState::Asking);

    let (take_picture, set_take_picture) = signal_local(None);
    let (close_camera, set_close_camera) = signal_local(None);

    let video_ref: NodeRef<html::Video> = NodeRef::new();
    let canvas_ref: NodeRef<html::Canvas> = NodeRef::new();
    let input_ref: NodeRef<html::Input> = NodeRef::new();

    let camera = move || {
        leptos::task::spawn_local(async move {
            use wasm_bindgen::{JsCast, JsValue};
            use wasm_bindgen_futures::JsFuture;

            gloo::timers::future::sleep(Duration::from_millis(500)).await;

            let canvas = match canvas_ref.get_untracked() {
                Some(canvas) => canvas,
                None => {
                    error!("canvas not found");
                    return;
                }
            };
            let video = match video_ref.get_untracked() {
                Some(video) => video,
                None => {
                    error!("video not found");
                    return;
                }
            };

            let window = match web_sys::window() {
                Some(window) => window,
                None => {
                    error!("window not found");
                    return;
                }
            };

            let camera = match window.navigator().media_devices() {
                Ok(media_devices) => media_devices,
                Err(e) => {
                    error!("Error getting media devices: {:?}", e);
                    return;
                }
            };

            let context = match canvas.get_context("2d") {
                Ok(Some(context)) => context,
                Ok(None) => {
                    error!("2d context not found");
                    return;
                }
                Err(e) => {
                    error!("error getting 2d context: {:?}", e);
                    return;
                }
            };
            let context = match context.dyn_into::<web_sys::CanvasRenderingContext2d>() {
                Ok(context) => context,
                Err(e) => {
                    error!("error casting 2d context: {:?}", e);
                    return;
                }
            };

            let camera_constraints = web_sys::MediaStreamConstraints::new();
            camera_constraints.set_video(&JsValue::from(true));
            camera_constraints.set_audio(&JsValue::from(false));

            let camera = match camera.get_user_media_with_constraints(&camera_constraints) {
                Ok(camera) => camera,
                Err(e) => {
                    error!("Error getting camera promise: {:?}", e);
                    return;
                }
            };
            set_camera_request_state(CameraRequestState::Asking);
            let camera = match JsFuture::from(camera).await {
                Ok(camera) => camera,
                Err(e) => {
                    set_camera_request_state(CameraRequestState::Denied);
                    error!("Error resolving camera future: {:?}", e);
                    return;
                }
            };
            set_camera_request_state(CameraRequestState::Granted);
            let camera = match camera.dyn_into::<MediaStream>() {
                Ok(camera) => camera,
                Err(e) => {
                    error!("Error mapping camera to object: {:?}", e);
                    return;
                }
            };

            video.set_src_object(Some(&camera));
            let promise = match video.play() {
                Ok(promise) => promise,
                Err(e) => {
                    error!("Error playing video: {:?}", e);
                    return;
                }
            };
            match JsFuture::from(promise).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Error resolving play promise: {:?}", e);
                    return;
                }
            };

            let capture = {
                let video = video.clone();
                Box::new(move || {
                    log!("Capturing image");
                    canvas.set_width(video.video_width());
                    canvas.set_height(video.video_height());
                    if let Err(e) = context.draw_image_with_html_video_element_and_dw_and_dh(
                        &video,
                        0.0,
                        0.0,
                        video.video_width() as f64,
                        video.video_height() as f64,
                    ) {
                        error!("Error drawing image: {:?}", e);
                    }

                    let data_url = match canvas.to_data_url_with_type("image/webp") {
                        Ok(data_url) => data_url,
                        Err(e) => {
                            error!("Error getting data url: {:?}", e);
                            return;
                        }
                    };
                    set_image_url(data_url.clone());
                })
            };

            let close_camera = Box::new(move || {
                log!("Closing camera");
                if let Err(e) = video.pause() {
                    error!("Error pausing video: {:?}", e);
                };
                video.set_src("");
                video.set_src_object(None);
                camera.get_tracks().iter().for_each(|track| {
                    let track = match track.dyn_into::<web_sys::MediaStreamTrack>() {
                        Ok(track) => track,
                        Err(e) => {
                            error!("Error casting track: {:?}", e);
                            return;
                        }
                    };
                    track.stop();
                });
            });

            set_take_picture(Some(capture));
            set_close_camera(Some(close_camera));
        })
    };

    {
        let jam_id = Rc::clone(&jam_id);
        Effect::new(move |_| {
            let jam_id: &str = &jam_id;
            let user_id: String = LocalStorage::get(jam_id).unwrap_or_default();
            if user_id.is_empty() {
                camera();
            } else if user_id == "kicked" {
                let navigate = use_navigate();
                navigate("/", NavigateOptions::default());
            } else {
                let navigate = use_navigate();
                navigate(&format!("/jam/{}", jam_id), NavigateOptions::default());
            }
        });
    }

    Effect::new(move |_| {
        if camera_request_state.with(|s| s.is_denied()) {
            if let Some(input) = input_ref.get() {
                file_picker(set_image_url, input);
            }
        }
    });
    let (name, set_name) = signal(String::from(""));

    let create_user = Action::new({
        let jam_id = (*jam_id).clone();
        move |_: &()| {
            let name = name.get();
            let pfp_url = image_url.get();
            let jam_id = (*jam_id).to_string();
            async move {
                if name.is_empty() {
                    return Err(ServerFnError::ServerError("Name is empty".into()));
                }
                create_user(jam_id, name, pfp_url).await
            }
        }
    });

    Effect::new(move |_| {
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

    Effect::new(move |_| {
        image_url.with(|url| {
            set_bg_img(url);
        })
    });

    view! {
        <div class="create-user">
            <div class="image-container">
                <video
                    style:display=move || {
                        if camera_request_state.with(|s| s.is_denied())
                            || image_url.with(|url| !url.is_empty())
                        {
                            "none"
                        } else {
                            "inline "
                        }
                    }

                    playsinline="false"
                    disablepictureinpicture
                    disableremoteplayback
                    node_ref=video_ref
                >
                    "Video stream not available."
                </video>
                <img
                    class="photo"
                    style:display=move || {
                        if !camera_request_state.with(|s| s.is_denied())
                            && image_url.with(|url| url.is_empty())
                        {
                            "none"
                        } else {
                            "inline"
                        }
                    }

                    prop:src=image_url
                    prop:alt=move || {
                        if camera_request_state.with(|s| s.is_denied()) {
                            "Click the circle to select an image"
                        } else {
                            "The screen capture will appear in this box."
                        }
                    }
                />

                <input
                    type="file"
                    node_ref=input_ref
                    name="image-picker"
                    id="image-picker"
                    accept=".webp, .png, .jpg, .gif, .jpeg"
                    multiple="false"
                    capture="user"
                />
                <canvas style:display="none" node_ref=canvas_ref></canvas>
            </div>
            <input
                type="text"
                class="text-input"
                placeholder="Name"
                class:glass-element-err=move || name.with(String::is_empty)
                on:input=move |ev| set_name(event_target_value(&ev))
            />
            <div class="buttons">
                <label
                    for="image-picker"
                    style:display=move || {
                        if image_url.with(|url| !url.is_empty())
                            || !camera_request_state.with(|s| s.is_denied())
                        {
                            "none"
                        } else {
                            "inline "
                        }
                    }
                >

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
                </label>

                <button
                    class="capture-button"
                    style:display=move || {
                        if image_url.with(|url| !url.is_empty())
                            || camera_request_state.with(|s| s.is_denied())
                        {
                            "none"
                        } else {
                            "inline "
                        }
                    }

                    on:click=move |_| {
                        if image_url.with(|url| url.is_empty()) {
                            if let Some(take) = take_picture() {
                                take();
                            }
                        }
                    }
                >

                    {move || {
                        if let CameraRequestState::Asking = camera_request_state.get() {
                            Either::Left(
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
                                },
                            )
                        } else {
                            Either::Right(
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
                                },
                            )
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
                        if let Some(close) = close_camera() {
                            close();
                        }
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

#[allow(dead_code)]
impl CameraRequestState {
    pub fn is_denied(&self) -> bool {
        matches!(self, CameraRequestState::Denied)
    }
    pub fn is_asking(&self) -> bool {
        matches!(self, CameraRequestState::Asking)
    }
    pub fn is_granted(&self) -> bool {
        matches!(self, CameraRequestState::Granted)
    }
}

///sets the image url every time the selected file changes
fn file_picker(image_url: WriteSignal<String>, file_input: HtmlInputElement) {
    let input = file_input;
    let listener = EventListener::new(&input, "change", {
        let input = input.clone();
        move |_| {
            let files = match input.files() {
                Some(files) => files,
                None => {
                    error!("no files found");
                    return;
                }
            };
            if files.length() > 0 {
                let file_reader = match FileReader::new() {
                    Ok(file_reader) => file_reader,
                    Err(e) => {
                        error!("error creating file reader: {:?}", e);
                        return;
                    }
                };
                let file = match files.item(0) {
                    Some(file) => file,
                    None => {
                        error!("no file found");
                        return;
                    }
                };
                if let Err(e) = file_reader.read_as_data_url(&file) {
                    error!("error reading file as data url: {:?}", e);
                    return;
                };
                let cb = {
                    let file_reader = file_reader.clone();
                    Closure::wrap(Box::new(move || {
                        let result = match file_reader.result() {
                            Ok(result) => result,
                            Err(e) => {
                                error!("error getting file reader result: {:?}", e);
                                return;
                            }
                        };
                        let url = match result.as_string() {
                            Some(url) => url,
                            None => {
                                error!("no url found");
                                return;
                            }
                        };
                        log!("file url:{}", url);
                        image_url(url);
                    }) as Box<dyn FnMut()>)
                };
                file_reader.set_onload(Some(cb.as_ref().unchecked_ref()));
                cb.forget();
            } else {
                image_url(String::new());
            }
        }
    });
    listener.forget();
}
