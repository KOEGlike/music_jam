use crate::app::general::*;
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};
use std::rc::Rc;

/// # panics
///     if both vote and remove songs are provided
#[component]
pub fn SongList(
    songs: ReadSignal<Vec<Song>>,
    votes: ReadSignal<Votes>,
    request_update: impl Fn() + 'static,
    #[prop(optional)] vote: Option<impl Fn(&str) + 'static>,
    #[prop(optional)] remove: Option<impl Fn(&str) + 'static>,
) -> impl IntoView {
    let local_songs:Vec<Song>;
    
    if vote.is_some() && remove.is_some() {
        panic!("You can't provide both vote and remove songs");
    }

    create_effect(move |_|{
        if songs().len() != votes().len() {
            request_update();
        }
    });


    let vote = Rc::new(vote);
    let remove = Rc::new(remove);

    view! {
        <div class="song-list">
            <For
                each=songs
                key=|song| song.id.clone()
                children=move |song| {
                    let vote = Rc::clone(&vote);
                    let remove = Rc::clone(&remove);
                    view! {
                        
                    }
                }
                />
        </div>}
}
