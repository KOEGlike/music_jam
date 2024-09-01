use crate::components::user::{get_width_of_element, millis_to_min_sec, will_element_overflow};
use crate::general::types::*;
use icondata::IoClose;
use leptos::{logging::*, prelude::*, *};

#[derive(Clone, Debug, Copy)]
pub enum SongAction {
    Vote {
        add_vote: Callback<String>,
        remove_vote: Callback<String>,
        vote: MaybeSignal<Vote>,
    },
    Remove {
        remove: Callback<String>,
        vote: MaybeSignal<Vote>,
    },
    Add(Callback<String>),
}

#[component]
pub fn Song(#[prop(optional_no_strip)] song: Option<Song>, song_type: SongAction) -> impl IntoView {
    let loaded = move |song: Song| {
        let title_id = song.id.clone() + "title";
        let artist_id = song.id.clone() + "artist";
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
                            SongAction::Remove { remove, .. } => remove(song_id.clone()),
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
                    <div class="info-text">
                        <div
                            class="title"
                            id=&title_id
                            class:scroll={
                                let title_id = title_id.clone();
                                move || {
                                    if cfg!(target_arch = "wasm32") {
                                        get_width_of_element(&title_id) > 130
                                    } else {
                                        false
                                    }
                                }
                            }
                        >

                            {move || {
                                let is_overflowing = if cfg!(target_arch = "wasm32") {
                                    get_width_of_element(&title_id) > 130
                                } else {
                                    false
                                };
                                std::iter::repeat(song.name.clone())
                                    .take(if is_overflowing { 2 } else { 1 })
                                    .collect::<Vec<String>>()
                                    .join(" ")
                            }}

                        </div>
                        <div class="small-info">
                            <div class="artist-wrapper" id="artist-wrapper">
                                <div
                                    class="artist"
                                    id=&artist_id
                                    class:scroll={
                                        let artist_id = artist_id.clone();
                                        move || {
                                            if cfg!(target_arch = "wasm32") {
                                                get_width_of_element(&artist_id) > 110
                                            } else {
                                                false
                                            }
                                        }
                                    }
                                >
                            
                                {move || {
                                    let artists = song.artists.join(", ");
                                    let is_overflowing = if cfg!(target_arch = "wasm32") {
                                        get_width_of_element(&artist_id) > 110
                                    } else {
                                        false
                                    };
                                    std::iter::repeat(artists)
                                        .take(if is_overflowing { 2 } else { 1 })
                                        .collect::<Vec<String>>()
                                        .join(" ")
                                }}
                                </div>
                            </div>
                            <span class="bullet-point">"•"</span>
                            <span class="song-duration">{millis_to_min_sec(song.duration)}</span>
                        </div>
                    </div>
                </div>

                <div class="action">

                    {match song_type {
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
                        SongAction::Remove { vote, .. } => {
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
                    }}

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
