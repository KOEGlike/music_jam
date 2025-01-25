use crate::model::types::Song;
use leptos::{
    either::Either,
    logging::{error, log},
    prelude::*,
    *,
};

#[component]
pub fn Player(
    #[prop(into)] position: Signal<f32>,
    #[prop(into)] current_song: ReadSignal<Option<Song>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    Effect::new(move |_| {
        current_song.with(|song| {
            if let Some(song) = song {
                set_bg_img(&song.image_url);
            }
        });
    });

    let song_length = move || current_song().map(|s| s.duration).unwrap_or_default();
    let (title_overflow, set_title_overflow) = signal(false);
    let (artist_overflow, set_artist_overflow) = signal(false);

    Effect::new(move |_| {
        current_song.track();
        set_artist_overflow(will_element_overflow("artist", Some("info")));
    });

    Effect::new(move |_| {
        current_song.track();
        set_title_overflow(will_element_overflow("title", Some("info")));
    });

    view! {
        <div class="player">
            <img
                prop:src=move || current_song().map(|s| s.image_url).unwrap_or_default()
                title="the album cover of the current song"
                alt="img not found, wait for a few seconds"
                onerror="this.src='data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw==';"
            />

            <div
                class="info"
                id="info"
                style:width={
                    let children_is_some = children.is_some();
                    move || if children_is_some { "320px" } else { "100%" }
                }
            >

                <div
                    class="title"
                    id="title"
                    class:scroll=move || {
                        current_song.track();
                        title_overflow()
                    }
                >

                    {move || { current_song().map(|s| s.name.clone()).unwrap_or_default() }}

                </div>
                <div
                    class="artist"
                    id="artist"
                    class:scroll=move || {
                        current_song.track();
                        artist_overflow()
                    }
                >

                    {move || { current_song().map(|s| s.artists.join(", ")).unwrap_or_default() }}

                </div>
            </div>

            <div class="progress">
                <div class="bar">
                    <div
                        class="position"
                        style:width=move || format!("{}%", position() * 100.0)
                    ></div>
                </div>
                <div class="times">
                    <div>
                        {move || millis_to_min_sec((position() * song_length() as f32) as u32)}
                    </div>
                    <div>{move || millis_to_min_sec(song_length())}</div>
                </div>
            </div>

            {if let Some(extra_elements) = children {
                Either::Left(extra_elements())
            } else {
                Either::Right(())
            }}

        </div>
    }
}

pub fn set_bg_img(url: &str) {
    match web_sys::window() {
        Some(window) => match window.document() {
            Some(document) => match document.document_element() {
                Some(element) => {
                    match element
                        .set_attribute("style", &format!("--background-url: url(\"{}\")", url))
                    {
                        Ok(_) => {
                            log!("background image set to {}", url);
                        }
                        Err(e) => {
                            error!("error setting background image: {:?}", e);
                        }
                    }
                }
                None => {
                    error!("document element not found");
                }
            },
            None => {
                error!("document not found");
            }
        },
        None => {
            error!("window not found");
        }
    }
}

pub fn will_element_overflow(element_id: &str, parent_id: Option<&str>) -> bool {
    use web_sys::*;
    let document = window().unwrap().document().unwrap();
    let element = match document.get_element_by_id(element_id) {
        Some(element) => element,
        None => {
            error!("element with id {} not found", element_id);
            return false;
        }
    };

    let parent_width = {
        if let Some(parent_id) = parent_id {
            match document.get_element_by_id(parent_id) {
                Some(element) => element,
                None => {
                    error!("element with id {} not found", parent_id);
                    return false;
                }
            }
        } else {
            element.parent_element().expect("the element has no parent")
        }
    }
    .client_width();

    let is_overflowing = parent_width < element.client_width();
    log!(
        "is_overflowing :{}, {} width:{}, {} width:{}",
        is_overflowing,
        element_id,
        element.scroll_width(),
        parent_id.unwrap_or("parent"),
        parent_width
    );
    is_overflowing
}

pub fn get_width_of_element(id: &str) -> i32 {
    use web_sys::*;
    let document = window().unwrap().document().unwrap();

    let width = match document.get_element_by_id(id) {
        Some(element) => element.scroll_width(),
        None => {
            error!("element with id {} not found", id);
            return 0;
        }
    };
    log!("width of element {} is {}", id, width);
    width
}

pub fn millis_to_min_sec(millis: u32) -> String {
    let seconds = millis / 1000;
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    format!("{:01}:{:02}", minutes, seconds)
}
