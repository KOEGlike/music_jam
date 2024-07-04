use crate::app::general::*;
use leptos::{logging::log, prelude::*, *};
use crate::app::components::general::Song;



#[component]
pub fn Search<F>(search_result: ReadSignal<Option<Vec<Song>>>, search:F) -> impl IntoView
where
    F: Fn(String),
{
    view! {
        <Song class="active" on_click=move || log!("Song clicked")>
            "lol"
        </Song>
    }
}
