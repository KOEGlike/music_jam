use crate::app::components::*;
use leptos::{logging::*, *};
use leptos_router::*;

#[component]
pub fn HomePage() -> impl IntoView{
    view! {
        <div id="home-page">
            <JoinIsland/>
            <CreateIsland/>
        </div>
    }
}