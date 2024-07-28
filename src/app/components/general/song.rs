use crate::app::general::types::*;
use icondata::IoClose;
use leptos::{prelude::*, *, logging::*};

#[derive(Clone, Debug, Copy)]
pub enum SongAction {
    Vote {
        add_vote: Callback<String>,
        remove_vote: Callback<String>,
        vote: MaybeSignal<Vote>,
    },
    Remove{
        remove: Callback<String>,
        vote: MaybeSignal<Vote>,
    },
    Add(Callback<String>),
}

#[component]
pub fn Song(
    #[prop(optional_no_strip)] song: Option<Song>,
    song_type: SongAction,
) -> impl IntoView {
    let loaded = move |song: Song| {
        view! {
            <div
                class="song"
                on:click={
                    let song_id = song.id.clone();
                    move |_| {
                        match song_type {
                            SongAction::Vote { add_vote, remove_vote, vote } => {
                                if let Some(vote) = vote().have_you_voted {
                                    
                                    if vote {
                                        log!("Removing vote");
                                        remove_vote(song_id.clone())
                                    } else {
                                        log!("Adding vote");
                                        add_vote(song_id.clone())
                                    }
                                }
                            }
                            SongAction::Remove{remove,..} => remove(song_id.clone()),
                            SongAction::Add(add) => add(song_id.clone()),
                        }
                    }
                }
            >

                <div class="info">
                    <img
                        src=&song.image_url
                        alt=format!("This is the album cover of {}", &song.name)
                    />
                    <div>
                        {&song.name}
                        <div>
                            {&song.artists.join(", ")} <span class="bullet-point">"•"</span>
                            <span class="song-duration">
                                {song.duration % 60} "." {song.duration / 60}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="action">

                    {
                        match song_type {
                            SongAction::Vote { vote, .. } => {
                                view! {
                                    <div class="votes">
                                        {move || vote().votes}
                                        <svg viewBox=IoClose.view_box inner_html=IoClose.data></svg>
                                    </div>
                                }
                                    .into_view()
                            }
                            SongAction::Add(_) => {
                                view! {
                                    <svg
                                        class="add"
                                        viewBox=IoClose.view_box
                                        inner_html=IoClose.data
                                    ></svg>
                                }
                                    .into_view()
                            }
                            SongAction::Remove{vote,..} => {
                                view! {
                                    {move || vote().votes}
                                    <svg
                                        class="remove"
                                        viewBox=IoClose.view_box
                                        inner_html=IoClose.data
                                    ></svg>
                                }
                                    .into_view()
                            }
                        }
                    }

                </div>
            </div>
        }
        .into_view()
    };

    let loading = move || view! {}.into_view();

    view! {
        {move || match song.clone() {
            Some(song) => loaded(song.clone()),
            None => loading(),
        }}
    }
}
