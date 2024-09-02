use crate::components::{Song, SongAction};
use crate::general::{types::real_time::SearchResult, *};
use icondata::AiSearchOutlined;
use leptos::{prelude::*, *};

#[component]
pub fn Search(
    #[prop(into)] search_result: Signal<Option<SearchResult>>,
    search: Callback<(String, String)>,
    add_song: Callback<String>,
    #[prop(into)] loaded: Signal<bool>,
) -> impl IntoView {
    let add_song = Callback::new(add_song);
    let search = Callback::new(search);
    let (current_result, set_current_result) = create_signal::<Vec<Song>>(Vec::new());
    let (search_id, set_search_id) = create_signal::<u128>(0);

    create_effect(move |_| {
        if let Some(search_result) = search_result() {
            if search_result.search_id == search_id().to_string() {
                set_current_result(search_result.songs.clone());
            }
        }
    });
    view! {
        <Show when=loaded fallback=move || "loading.s.....">
            <div class="search">
                <div class="search-input">
                    <input
                        type="text"
                        placeholder="Search for a song"
                        on:input=move |ev| {
                            let id = search_id.get_untracked();
                            search((event_target_value(&ev), id.to_string()));
                        }
                    />

                    <button>
                        <svg
                            inner_html=AiSearchOutlined.data
                            viewBox=AiSearchOutlined.view_box
                        ></svg>
                    </button>
                </div>
                <div class="search-result">
                    <For
                        each=move || current_result().into_iter()
                        key=|song| song.id.clone()
                        children=move |song| {
                            view! {
                                <Song song=Some(song.clone()) song_type=SongAction::Add(add_song)/>
                            }
                        }
                    />

                </div>
            </div>
        </Show>
    }
}
