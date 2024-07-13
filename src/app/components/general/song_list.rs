use crate::app::general::*;
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};
use std::{borrow::Borrow, rc::Rc};

#[derive(Clone, Debug)]
pub enum SongAction<F>
where
    F: Fn(&str) + Clone + 'static,
{
    Vote(F),
    Remove(F),
}

#[component]
pub fn SongList(
    songs: impl Fn() -> Vec<Song> + 'static,
    votes: impl Fn() -> Votes + 'static,
    request_update: impl Fn() + 'static,
    song_action: SongAction<impl Fn(&str) + Clone + 'static>,
) -> impl IntoView {
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

    let song_action = Rc::new(song_action);

    view! {
        <div class="song-list">
            <For
                each=songs
                key=|song| song.id.clone()
                children=move |song| {
                    let song_action = Rc::clone(&song_action);
                    view! {
                        <div on:click={
                            let song_action = Rc::clone(&song_action);
                            let song_id = song.id.clone();
                            move |_| {
                                match song_action.borrow() {
                                    SongAction::Vote(vote) => vote(&song_id),
                                    SongAction::Remove(remove) => remove(&song_id),
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
                                let song_action = Rc::clone(&song_action);
                                let song_id = song.id.clone();
                                match song_action.borrow() {
                                    SongAction::Vote(_) => song.votes.into_view(),
                                    SongAction::Remove(remove_song) => {
                                        let remove_song = remove_song.clone();
                                        view! {
                                            <button
                                                class="remove-song"
                                                on:click=move |_| {
                                                    remove_song(&song_id);
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
