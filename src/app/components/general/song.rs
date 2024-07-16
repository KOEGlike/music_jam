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
pub fn Song(song: MaybeSignal<Option<Song>>, song_action: SongAction) -> impl IntoView {
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

                <div class="song-info">
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

                <div class="song-action">

                    {
                        let song = song.clone();
                        let song_id = song.id.clone();
                        match song_action {
                            SongAction::Vote(_) => song.votes.into_view(),
                            SongAction::Add(_) => "+".into_view(),
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

            </div>
        }
        .into_view()
    };

    let loading = move || view! {}.into_view();

    view! {
        {move || match song() {
            Some(song) => loaded(song.clone()),
            None => loading(),
        }}
    }
}
