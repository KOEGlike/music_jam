use crate::general::*;
use leptos::{logging::log, prelude::*, *};
use std::rc::Rc;

#[component]
pub fn YourSongs<F>(
    songs: ReadSignal<Vec<Song>>,
    max_song_count: u8,
    remove_song: F,
) -> impl IntoView
where
    F: Fn(&str) + 'static,
{
    let songs = move || {
        songs()
            .iter()
            .filter(|song| song.user_id.is_some())
            .cloned()
            .collect::<Vec<Song>>()
    };

    let remove_song= Rc::new(remove_song);

    view! {
        <div class="your-songs">
            <div>
                <For
                    each=songs
                    key=|song| song.id.clone()
                    children=move |song| {
                        let remove_song = Rc::clone(&remove_song);
                        view! {
                            <button on:click=move |_| {
                                remove_song(&song.id);
                            }>
                                <img
                                    src=&song.image_url
                                    alt=format!("This is the album cover of {}", &song.name)
                                    title=&song.name
                                />
                            </button>
                        }
                    }
                />

            </div>
            {songs().len()}
            /
            {max_song_count}
        </div>
    }
}
