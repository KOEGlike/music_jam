use crate::components::*;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView{
    view! {
        <div class="home-page">
            <JoinIsland/>
            <CreateIsland/>
        </div>
    }
}