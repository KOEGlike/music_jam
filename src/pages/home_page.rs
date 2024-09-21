use crate::components::*;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView{
    view! {
        <div id="home-page">
            <JoinIsland/>
            <CreateIsland/>
        </div>
    }
}