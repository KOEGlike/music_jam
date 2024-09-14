use crate::components::{Song, SongAction};
use crate::model::{types::real_time::SearchResult, *};
use icondata::AiSearchOutlined;
use leptos::{prelude::*, *};
use logging::log;

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
    let add_song = Callback::new(move |id| {
        set_current_result(vec![]);
        add_song(id);
    });

    create_effect(move |_| {
        if let Some(search_result) = search_result() {
            set_current_result(search_result.songs.clone());
        }
    });
    
    view! {
        <div class="search">
            <div class="search-input">
                <input
                    type="text"
                    placeholder="Search for a song"
                    on:input=move |ev| {
                        if loaded.get_untracked() {
                            let id = "";
                            search((event_target_value(&ev), id.to_string()));
                        }
                    }
                />

                <button class:loaded=loaded>
                    {move || {
                        if loaded() {
                            view! {
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    inner_html=AiSearchOutlined.data
                                    viewBox=AiSearchOutlined.view_box
                                ></svg>
                            }
                        } else {
                            view! {
                                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                                    <circle cx="18" cy="12" r="0">
                                        <animate
                                            attributeName="r"
                                            begin=".67"
                                            calcMode="spline"
                                            dur="1.5s"
                                            keySplines="0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8"
                                            repeatCount="indefinite"
                                            values="0;2;0;0"
                                        ></animate>
                                    </circle>
                                    <circle cx="12" cy="12" r="0">
                                        <animate
                                            attributeName="r"
                                            begin=".33"
                                            calcMode="spline"
                                            dur="1.5s"
                                            keySplines="0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8"
                                            repeatCount="indefinite"
                                            values="0;2;0;0"
                                        ></animate>
                                    </circle>
                                    <circle cx="6" cy="12" r="0">
                                        <animate
                                            attributeName="r"
                                            begin="0"
                                            calcMode="spline"
                                            dur="1.5s"
                                            keySplines="0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8;0.2 0.2 0.4 0.8"
                                            repeatCount="indefinite"
                                            values="0;2;0;0"
                                        ></animate>
                                    </circle>
                                </svg>
                            }
                        }
                    }}

                </button>
            </div>
            <div class="search-result">
                <For
                    each=move | | current_result().into_iter()
                    key=|song| song.spotify_id.clone()
                    children=move |song| {
                        view! {
                            <Song song=Some(song.clone()) song_type=SongAction::Add(add_song)/>
                        }
                    }
                />

            </div>
        </div>
    }
}
