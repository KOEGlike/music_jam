use crate::components::user::{get_width_of_element, millis_to_min_sec, will_element_overflow};
use crate::model::types::*;
use icondata::IoClose;
use leptos::{
    either::{Either, EitherOf3},
    logging::*,
    prelude::*,
    *,
};
use std::f32::consts::E;
use std::rc::Rc;

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

impl SongAction {
    pub fn is_vote(&self) -> bool {
        matches!(self, SongAction::Vote { .. })
    }
    pub fn is_remove(&self) -> bool {
        matches!(self, SongAction::Remove { .. })
    }
    pub fn is_add(&self) -> bool {
        matches!(self, SongAction::Add(_))
    }
}

#[component]
pub fn Song(#[prop(optional_no_strip)] song: Option<Song>, song_type: SongAction) -> impl IntoView {
    let loaded = move |song: Song| {
        let id_ref: NodeRef<html::Div> = NodeRef::new();
        let title_ref: NodeRef<html::Div> = NodeRef::new();
        let artist_ref: NodeRef<html::Div> = NodeRef::new();

        let mut width: u16 = 180;
        if song_type.is_add() {
            width = 150;
        }

        let title_overflowing = move || {
            let title_width = title_ref.get().unwrap().client_width();
            title_width > width as i32
        };
        let artist_overflowing = move || {
            let artist_width = artist_ref.get().unwrap().client_width();
            artist_width > width as i32
        };

        view! {
            <div
                class="song"
                title=song.name
                class:voted=move || {
                    if let SongAction::Vote { vote, .. } = song_type {
                        vote().have_you_voted.unwrap_or(false)
                    } else {
                        false
                    }
                }

                class:remove=song_type.is_remove()
                on:click={
                    let spotify_song_id = song.spotify_id.clone();
                    let song_id = song.id.clone().unwrap_or_default();
                    move |_| {
                        match song_type {
                            SongAction::Vote { add_vote, remove_vote, vote } => {
                                if let Some(vote) = vote().have_you_voted {
                                    if vote {
                                        log!("Removing vote");
                                        remove_vote.run(song_id.clone())
                                    } else {
                                        log!("Adding vote");
                                        add_vote.run(song_id.clone())
                                    }
                                }
                            }
                            SongAction::Remove { remove, .. } => remove.run(song_id.clone()),
                            SongAction::Add(add) => add.run(spotify_song_id.clone()),
                        }
                    }
                }
            >

                {move || {
                    if song_type.is_remove() {
                        Either::Left(
                            view! {
                                <svg
                                    class="remove"
                                    viewBox=IoClose.view_box
                                    inner_html=IoClose.data
                                ></svg>
                            },
                        )
                    } else {
                        Either::Right(())
                    }
                }}

                <div class="info" id="info">
                    <img
                        src=song.image_url
                        alt=format!("This is the album cover of {}", song.name)
                    />
                    <div
                        class="info-text"
                        id="info-txt"
                        style:width=move || { format!("{}px", width) }
                        node_ref=id_ref
                    >
                        <div
                            class="title"
                            node_ref=title_ref

                            class:scroll=title_overflowing
                        >
                            {song.name.clone()}
                        </div>
                        <div class="small-info">
                            <div class="artist-wrapper" id="artist-wrapper">
                                <div
                                    class="artist"
                                    node_ref=artist_ref

                                    class:scroll=artist_overflowing
                                >
                                    {move || {
                                        let artists = song.artists.join(", ");
                                        std::iter::repeat(artists)
                                            .take(if artist_overflowing() { 2 } else { 1 })
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
                            EitherOf3::A(view! { <div class="votes">{move || vote().votes}</div> })
                        }
                        SongAction::Add(_) => {
                            EitherOf3::B(
                                view! {
                                    <svg
                                        class="add"
                                        viewBox=IoClose.view_box
                                        inner_html=IoClose.data
                                    ></svg>
                                },
                            )
                        }
                        SongAction::Remove { vote, .. } => {
                            EitherOf3::C(view! { {move || vote().votes} })
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
            Some(song) => Either::Left(loaded(song)),
            None => Either::Right(loading()),
        }}
    }
}
