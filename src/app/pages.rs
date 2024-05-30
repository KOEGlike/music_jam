use leptos::*;
use leptos_router::*;
use crate::app::components::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div id="home-page">
            <JoinIsland/>
            <CreateIsland/>
        </div>
    }
}
