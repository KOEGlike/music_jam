use crate::app::general;
use crate::components::{Song, SongAction};
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn SongList<F>(
    songs: ReadSignal<Option<Vec<general::Song>>>,
    votes: ReadSignal<general::Votes>,
    request_update: F,
    song_action: SongAction,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let songs_vec = move || {
        if let Some(songs) = songs() {
            let votes = votes();
            if songs.len() != votes.len() {
                request_update();
                return songs;
            }
            let songs = songs
                .iter()
                .map(|song| {
                    let votes = votes.get(&song.id).copied().unwrap_or(0);
                    general::Song {
                        votes,
                        ..song.clone()
                    }
                })
                .collect::<Vec<_>>();
            songs
        } else {
            Vec::new()
        }
    };

    view! {
        {move || {
            if songs().is_none() {
                let mut vec = Vec::new();
                for _ in 0..5 {
                    vec.push(
                        view! { <Song song=MaybeSignal::Static(None) song_action=song_action/> },
                    );
                }
                vec.into_view()
            } else {
                ().into_view()
            }
        }}

        <div class="song-list">
            <For
                each=songs_vec
                key=|song| song.id.clone()
                children=move |song| {
                    view! {
                        <Song song=MaybeSignal::Static(Some(song.clone())) song_action=song_action/>
                    }
                }
            />

        </div>
    }
}
