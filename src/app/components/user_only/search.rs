use crate::app::components::{Song, SongAction};
use crate::app::general::*;
use icondata::AiSearchOutlined;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn Search(
    #[prop(into)] search_result: Signal<Option<Vec<Song>>>,
    search: Callback<String>,
    add_song: Callback<String>,
) -> impl IntoView

{
    let add_song = Callback::new(add_song);
    let search = Callback::new(search);
    view! {
        <Show when=move || search_result.with(Option::is_some) fallback=move || "loading.s.....">
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
                        each=move || search_result().unwrap_or_default()
                        key=|song| song.id.clone()
                        children=move |song| {
                            view! {
                                <Song
                                    song=Some(song.clone())
                                    song_type=SongAction::Add(add_song)
                                />
                            }
                        }
                    />

                </div>
            </div>
        </Show>
    }
}
