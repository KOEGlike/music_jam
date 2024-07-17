use crate::app::components::{Song, SongAction};
use crate::app::general::*;
use icondata::AiSearchOutlined;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn Search<F>(
    #[prop(into)] search_result: Signal<Vec<Song>>,
    search: F,
    add_song: F,
) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    let add_song = Callback::new(add_song);
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
                            <Song
                                song=Some(song.clone())
                                song_action=SongAction::Add(add_song)
                            />
                        }
                    }
                />

            </div>
        </div>
    }
}
