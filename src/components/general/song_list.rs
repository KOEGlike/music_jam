use crate::components::{Song, SongAction};
use crate::model::{self, Vote, *};
use leptos::{either::Either, logging::log, prelude::*};

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

impl SongListAction {
    pub fn is_vote(&self) -> bool {
        matches!(self, SongListAction::Vote { .. })
    }
    pub fn is_remove(&self) -> bool {
        matches!(self, SongListAction::Remove(_))
    }
    pub fn is_add(&self) -> bool {
        matches!(self, SongListAction::Add(_))
    }
}

#[component]
pub fn SongList(
    #[prop(into)] songs: Signal<Option<Vec<model::Song>>>,
    #[prop(into)] votes: Signal<model::Votes>,
    #[prop(into)] max_song_count: Signal<u8>,
    song_list_action: SongListAction,
) -> impl IntoView {
    let (button_state, set_button_state) = signal(false);
    let songs = move || {
        if let Some(songs) = songs.get() {
            let votes = votes.get();
            if songs.len() != votes.len() {
                return Some(songs);
            }
            let mut songs = songs
                .into_iter()
                .map(|mut song| {
                    let votes = votes
                        .get(song.id.as_deref().unwrap_or(""))
                        .copied()
                        .unwrap_or(Vote {
                            votes: 0,
                            have_you_voted: None,
                        });
                    song.votes = votes;
                    song
                })
                .collect::<Vec<_>>();
            if let SongListAction::Remove(_) = song_list_action {
                songs.sort_by_key(|song| song.votes.votes);
            }
            let songs = songs.into_iter().rev().collect::<Vec<_>>();
            Some(songs)
        } else {
            None
        }
    };
    let songs = Signal::derive(songs);

    let your_songs = Signal::derive(move || {
        songs.get().map(|songs| {
            songs
                .into_iter()
                .filter(|song| song.user_id.is_some())
                .collect::<Vec<types::Song>>()
        })
    });
    let others_songs = Signal::derive(move || {
        songs.get().map(|songs| {
            songs
                .into_iter()
                .filter(|song| song.user_id.is_none())
                .collect::<Vec<types::Song>>()
        })
    });

    view! {
        <div class="song-list">
            {if song_list_action.is_vote() {
                Either::Left(
                    view! {
                        <div class="header">
                            <button
                                class="vote"
                                on:click=move |_| set_button_state.set(false)
                                class:active=move || !button_state.get()
                            >
                                "Vote"
                            </button>
                            <button
                                class="add"
                                on:click=move |_| set_button_state.set(true)
                                class:active=button_state
                            >
                                {move || {
                                    format!(
                                        "Your ({} / {})",
                                        your_songs.get().unwrap_or_default().len(),
                                        max_song_count.get(),
                                    )
                                }}

                            </button>
                        </div>
                    },
                )
            } else {
                Either::Right(())
            }}
            <div class="songs">
                {move || {
                    if songs.with(|songs| { songs.is_none() }) {
                        let mut vec = Vec::new();
                        for _ in 0..5 {
                            vec.push(
                                view! {
                                    <Song
                                        song=None
                                        song_type=SongAction::Add(Callback::new(|_| {}))
                                    />
                                },
                            );
                        }
                        Either::Left(vec.into_view())
                    } else {
                        Either::Right(())
                    }
                }}
                {move || {
                    match button_state.get() {
                        false => {
                            Either::Left(
                                view! {
                                    <div>
                                        {move || {
                                            if let Some(songs) = songs.get() {
                                                if songs.is_empty() {
                                                    return Either::Left(
                                                        view! {
                                                            <div class="no-songs">"No songs in this jam 😔"</div>
                                                        },
                                                    );
                                                }
                                            }
                                            Either::Right(())
                                        }}
                                        <For
                                            each=move || {
                                                others_songs.get().unwrap_or_default().into_iter()
                                            }

                                            key=|song| song.id.clone()
                                            children=move |song| {
                                                let id = song.id.clone();
                                                let votes = Memo::new(move |_| {
                                                    others_songs
                                                        .with(|songs| {
                                                            {
                                                                songs
                                                                    .as_ref()
                                                                    .map(|songs| {
                                                                        songs
                                                                            .iter()
                                                                            .filter(|s| s.id == id)
                                                                            .map(|s| s.votes)
                                                                            .next()
                                                                            .unwrap_or(Vote {
                                                                                votes: 69,
                                                                                have_you_voted: None,
                                                                            })
                                                                    })
                                                            }
                                                                .unwrap_or_default()
                                                        })
                                                });
                                                let name = song.name.clone();
                                                Effect::new(move |_| {
                                                    log!("votes: {:#?}, song name:{}", votes.get(), name);
                                                });
                                                let song_action = match song_list_action {
                                                    SongListAction::Vote { add_vote, remove_vote, .. } => {
                                                        SongAction::Vote {
                                                            add_vote,
                                                            remove_vote,
                                                            vote: votes.into(),
                                                        }
                                                    }
                                                    SongListAction::Remove(cb) => {
                                                        SongAction::Remove {
                                                            remove: cb,
                                                            vote: votes.into(),
                                                        }
                                                    }
                                                    SongListAction::Add(cb) => SongAction::Add(cb),
                                                };
                                                view! { <Song song=Some(song) song_type=song_action /> }
                                            }
                                        />

                                    </div>
                                },
                            )
                        }
                        true => {
                            Either::Right(

                                view! {
                                    <div>
                                        {move || {
                                            if let Some(songs) = your_songs.get() {
                                                if songs.is_empty() {
                                                    return Either::Left(
                                                        view! {
                                                            <div class="no-songs">
                                                                "You didn't add any songs yet😔"<br />
                                                                " search for something to add a song"
                                                            </div>
                                                        },
                                                    );
                                                }
                                            }
                                            Either::Right(())
                                        }}
                                        <For
                                            each=move || {
                                                your_songs.get().unwrap_or_default().into_iter()
                                            }

                                            key=|song| song.id.clone()
                                            children=move |song| {
                                                if let SongListAction::Vote { remove_song, .. } = song_list_action {
                                                    let id = song.id.clone();
                                                    let votes = Memo::new(move |_| {
                                                        your_songs
                                                            .with(|songs| {
                                                                {
                                                                    songs
                                                                        .as_ref()
                                                                        .map(|songs| {
                                                                            songs
                                                                                .iter()
                                                                                .filter(|s| s.id == id)
                                                                                .map(|s| s.votes)
                                                                                .next()
                                                                                .unwrap_or(Vote {
                                                                                    votes: 69,
                                                                                    have_you_voted: None,
                                                                                })
                                                                        })
                                                                }
                                                                    .unwrap_or_default()
                                                            })
                                                    });
                                                    let song_action = SongAction::Remove {
                                                        remove: remove_song,
                                                        vote: votes.into(),
                                                    };
                                                    Either::Left(
                                                        view! { <Song song=Some(song) song_type=song_action /> },
                                                    )
                                                } else {
                                                    Either::Right(())
                                                }
                                            }
                                        />

                                    </div>
                                },
                            )
                        }
                    }
                }}
            </div>

        </div>
    }
}
