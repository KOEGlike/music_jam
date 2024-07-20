use crate::app::general::types::*;
use icondata::IoClose;
use leptos::{prelude::*, *};

#[derive(Clone, Debug, Copy)]
pub enum SongAction {
    Vote(Callback<String>),
    Remove(Callback<String>),
    Add(Callback<String>),
}

#[component]
pub fn Song(
    #[prop(optional_no_strip)] song: Option<Song>,
    #[prop(into)]
    #[prop(optional)]
    votes: Option<MaybeSignal<u32>>,
    song_action: SongAction,
) -> impl IntoView {
    let loaded = move |song: Song| {
        view! {
            <div
                class="song"
                on:click={
                    let song_id = song.id.clone();
                    move |_| {
                        match song_action {
                            SongAction::Vote(vote) => vote(song_id.clone()),
                            SongAction::Remove(remove) => remove(song_id.clone()),
                            SongAction::Add(add) => add(song_id.clone()),
                        }
                    }
                }
            >
                <div class="info">
                    <img
                        src=&song.image.url
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
                        let song = song.clone();
                        match song_action {
                            SongAction::Vote(_) => {
                                let votes = if let Some(votes) = votes {
                                    votes()
                                } else {
                                    song.votes as u32
                                };
                                view! {
                                    <div class="votes">
                                        {votes}
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
                            SongAction::Remove(_) => {
                                view! {
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
