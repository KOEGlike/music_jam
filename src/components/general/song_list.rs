use crate::general::{self, Vote};
use crate::components::{Song, SongAction};
use leptos::{logging::log, prelude::*, *};

#[derive(Clone, Debug, Copy)]
pub enum SongListAction {
    Vote {
        add_vote: Callback<String>,
        remove_vote: Callback<String>,
        remove_song: Callback<String>,
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
                .filter(|song|song.user_id.is_none())
                .map(|song| {
                    let votes = votes.get(&song.id).copied().unwrap_or(Vote {
                        votes: 0,
                        have_you_voted: None,
                    });
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
    create_effect(move|_|{
        log!("votes: {:#?}",votes());
    });

    view! {
        <div class="song-list">
            {move || {
                if songs.with(|songs| { songs.is_none() }) {
                    let mut vec = Vec::new();
                    for _ in 0..5 {
                        vec.push(
                            view! {
                                <Song song=None song_type=SongAction::Add(Callback::new(|_| {}))/>
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
                                        .map(|songs| {
                                            songs.get(index).map(|s| s.votes).unwrap_or_default()
                                        })
                                }
                                    .unwrap_or_default()
                            })
                    });
                    let song_action = match song_list_action {
                        SongListAction::Vote { add_vote, remove_vote, .. } =>
                            SongAction::Vote {
                                add_vote,
                                remove_vote,
                                vote: votes.into(),
                            },

                        SongListAction::Remove(cb) => SongAction::Remove{remove:cb, vote:votes.into()},
                        SongListAction::Add(cb) => SongAction::Add(cb),
                    };
                    view! { <Song song=Some(song) song_type=song_action/> }
                }
            />

        </div>
    }
}
