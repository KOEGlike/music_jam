use leptos::*;
use leptos_router::*;
use crate::app::components::*;

pub fn HomePage() -> impl IntoView {
    view! {
        <div>
            <JoinIsland/>
        </div>
    }
}