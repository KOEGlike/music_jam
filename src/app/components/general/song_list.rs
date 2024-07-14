use crate::app::general;
use crate::components::{Song, SongAction};
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn SongList<F>(
    songs: ReadSignal<Vec<general::Song>>,
    votes: ReadSignal<general::Votes>,
    request_update: F,
    song_action: SongAction,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let songs = move || {
        let songs = songs();
        let votes = votes();
        if songs.len() != votes.len() {
            request_update();
            return songs;
        }
        songs
            .iter()
            .map(|song| {
                let votes = votes.get(&song.id).copied().unwrap_or(0);
                general::Song {
                    votes,
                    ..song.clone()
                }
            })
            .collect::<Vec<_>>()
    };

    view! {
        <div class="song-list">
            <For
                each=songs
                key=|song| song.id.clone()
                children=move |song| {
                    view! { <Song song=song.clone() song_action=song_action.clone()/> }
                }
            />

        </div>
    }
}
