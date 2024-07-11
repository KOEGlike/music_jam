use leptos_router::*;
use leptos::{logging::log, prelude::*, *};
use gloo::storage::{LocalStorage, Storage};
use crate::app::components::host_only::Player;

#[component]
pub fn HostPage() -> impl IntoView {
    let (host_id, set_host_id) = create_signal(String::new());

    create_effect(move |_| {
        set_host_id(LocalStorage::get("host_id").unwrap());
    });

    view!{
        {
            move||{
                if !host_id().is_empty() {
                    log!("host_id: {}", host_id());
                    view!{<Player host_id={host_id()}/>}.into_view()
                } else {
                    "loading....".into_view()
                }
            }
        }
    }
}