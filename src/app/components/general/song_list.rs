use crate::app::general;
use crate::components::{Song, SongAction, SongVoteState};
use leptos::{logging::log, prelude::*, *};
use std::collections::HashMap;

#[derive(Clone, Debug, Copy)]
pub enum SongListAction {
    Vote {
        add_vote: Callback<String>,
        remove_vote: Callback<String>,
        user_votes: Signal<HashMap<String, Signal<SongVoteState>>>,
    },
    Remove(Callback<String>),
    Add(Callback<String>),
}

#[component]
pub fn SongList<F>(
    #[prop(into)] songs: Signal<Option<Vec<general::Song>>>,
    #[prop(into)] votes: Signal<general::Votes>,
    request_update: F,
    song_list_action: SongListAction,
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
                                <Song song=None song_action=SongAction::Add(Callback::new(|_| {}))/>
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
                key=|(_, song)| song.id.clone()
                children=move |(index, song)| {
                    let votes = create_memo(move |_| {
                        songs
                            .with(|songs| {
                                {
                                    songs
                                        .as_ref()
                                        .map(|songs| songs.get(index).map(|s| s.votes).unwrap_or(0))
                                }
                                    .unwrap_or(0) as u32
                            })
                    });
                    let song_action = match song_list_action {
                        SongListAction::Vote { add_vote, remove_vote, user_votes } => {
                            let song_id = song.id.clone();
                            SongAction::Vote {
                                add_vote,
                                remove_vote,
                                current_state: user_votes()
                                    .get(&song_id)
                                    .cloned()
                                    .unwrap_or(Signal::derive(move || SongVoteState::Loading)),
                            }
                        }
                        SongListAction::Remove(cb) => SongAction::Remove(cb),
                        SongListAction::Add(cb) => SongAction::Add(cb),
                    };
                    view! { <Song song=Some(song) song_action=song_action votes=votes/> }
                }
            />

        </div>
    }
}
