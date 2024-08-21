use crate::app::components::{Song, SongAction};
use crate::general::*;
use icondata::AiSearchOutlined;
use leptos::{prelude::*, *};

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
                <div class="search-input">
                    <input
                        type="text"
                        placeholder="Search for a song"
                        on:input=move |ev| {
                            search(event_target_value(&ev));
                        }
                    />

                    <button>
                        <svg inner_html=AiSearchOutlined.data viewBox=AiSearchOutlined.view_box></svg>
                    </button>
                </div>
                <div class="search-result">
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
