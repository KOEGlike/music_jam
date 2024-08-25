use crate::components::*;
use leptos::*;

#[component]
pub fn HomePage() -> impl IntoView{
    view! {
        <div id="home-page">
            <JoinIsland/>
            <CreateIsland/>
        </div>
    }
}