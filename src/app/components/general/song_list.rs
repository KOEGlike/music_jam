use crate::app::general::*;
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};
use std::{borrow::Borrow, rc::Rc};

#[derive(Clone, Debug)]
pub enum SongAction {
    Vote(Callback<String>),
    Remove(Callback<String>),
}

#[component]
pub fn SongList<F>(
    songs: ReadSignal<Vec<Song>>,
    votes: ReadSignal<Votes>,
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
                Song {
                    votes,
                    ..song.clone()
                }
            })
            .collect::<Vec<_>>()
    };

    //let song_action = Rc::new(song_action);

    view! {
        <div class="song-list">
            <For
                each=songs
                key=|song| song.id.clone()
                children=move |song| {
                    view! {
                        // let song_action = Rc::clone(&song_action);
                        <div on:click={
                            let song_action = song_action.clone();
                            let song_id = song.id.clone();
                            move |_| {
                                match song_action.clone() {
                                    SongAction::Vote(vote) => vote(song_id.clone()),
                                    SongAction::Remove(remove) => remove(song_id.clone()),
                                }
                            }
                        }>

                            <div>
                                <img
                                    src=&song.image.url
                                    alt=format!("This is the album cover of {}", &song.name)
                                />
                                <div>
                                    {&song.name}
                                    <div>
                                        {&song.artists.join(", ")} "·" {song.duration % 60} "."
                                        {song.duration / 60}
                                    </div>
                                </div>
                            </div>

                            {
                                let song = song.clone();
                                let song_id = song.id.clone();
                                match song_action {
                                    SongAction::Vote(_) => song.votes.into_view(),
                                    SongAction::Remove(remove_song) => {
                                        view! {
                                            <button
                                                class="remove-song"
                                                on:click=move |_| {
                                                    remove_song(song_id.clone());
                                                }
                                            >

                                                <svg viewBox=IoClose.view_box inner_html=IoClose.data></svg>
                                            </button>
                                        }
                                            .into_view()
                                    }
                                }
                            }

                        </div>
                    }
                }
            />

        </div>
    }
}
