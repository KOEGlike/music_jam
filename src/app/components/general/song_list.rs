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
                each=move || songs().unwrap_or_default().into_iter().enumerate()
                key=|(_,song)| song.id.clone()
                children=move |(index, song)| {
                    let votes=create_memo(move|_|songs.with(|songs|{songs.as_ref().map(|songs|songs.get(index).map(|s|s.votes).unwrap_or(0))}.unwrap_or(0)));
                    let song=move||{
                        let mut=song.clone();
                        song.votes=votes();
                        Some(song)
                    };
                    view! {
                        "lol"
                    }
                }
            />

        </div>
    }
}
