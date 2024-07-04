use crate::app::general::*;
use icondata::AiSearchOutlined;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn Search<F1,F2>(search_result: ReadSignal<Vec<Song>>, search: F1, add_song:F2) -> impl IntoView
where
    F1: Fn(&str) + 'static,
    F2: Fn(&str) + 'static,
{
    view! {
        <div class="search">
            <div>
                <input
                    type="text"
                    placeholder="Search for a song"
                    on:input=move |ev| {
                        search(&event_target_value(&ev));
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
                            <div on:click=|_| {}>
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
                            </div>
                        }
                    }
                />

            </div>
        </div>
    }
}
