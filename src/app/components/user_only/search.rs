use crate::app::general::*;
use icondata::AiSearchOutlined;
use leptos::{logging::log, prelude::*, *};
use crate::app::components::{Song, SongAction};

#[component]
pub fn Search<F1,F2>(search_result: ReadSignal<Vec<Song>>, search: F1, add_song:F2) -> impl IntoView
where
    F1: Fn(String) + 'static,
    F2: Fn(String) + 'static,
{

    let add_song=Callback::new(add_song);
    view! {
        <div class="search">
            <div>
                <input
                    type="text"
                    placeholder="Search for a song"
                    on:input=move |ev| {
                        search(event_target_value(&ev));
                    }
                />

                <button>
                    <svg></svg>
                </button>
            </div>
            <div>
                <For
                    each=search_result
                    key=|song| song.id.clone()
                    children=move |song| {
                        view! {
                            <Song song=song.clone() song_action=SongAction::Add(add_song)/>
                        }
                    }
                />

            </div>
        </div>
    }
}
