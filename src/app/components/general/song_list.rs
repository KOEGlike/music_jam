use crate::app::general;
use crate::components::{Song, SongAction};
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn SongList<F>(
    #[prop(into)] songs: Signal<Option<Vec<general::Song>>>,
    #[prop(into)] votes: Signal<general::Votes>,
    request_update: F,
    song_action: SongAction,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let songs = move || {
        if let Some(songs) = songs() {
            let votes = votes();
            if songs.len() != votes.len() {
                request_update();
                return Some(songs);
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
            Some(songs)
        } else {
            None
        }
    };

    let songs = Signal::derive(songs);

    view! {
        <div class="song-list">
            {move || {
                if songs.with(|songs| { songs.is_none() }) {
                    let mut vec = Vec::new();
                    for _ in 0..5 {
                        vec.push(
                            view! {
                                // avoids cloning the whole list

                                <Song song=None song_action=song_action/>
                            },
                        );
                    }
                    vec.into_view()
                } else {
                    ().into_view()
                }
            }}
            <For
                each=move || songs().unwrap_or_default()
                key=|song| song.id.clone()
                children=move |song| {
                    view! {
                        <Song song=Some(song.clone()) song_action=song_action/>
                    }
                }
            />

        </div>
    }
}
