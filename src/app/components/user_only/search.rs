use crate::app::general::*;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn Search<F>(search_result: ReadSignal<Option<Vec<Song>>>, search:F) -> impl IntoView
where
    F: Fn(String),
{
    
}
