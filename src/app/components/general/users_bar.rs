use std::result;

use crate::app::general::*;
use gloo::{history::query, utils::format};
use leptos::{logging::log, prelude::*, *};


#[component]
pub fn UsersBar(
    jam_id: MaybeSignal<String>,
    #[prop(optional_no_strip)] host_id: Option<MaybeSignal<String>>,
) -> impl IntoView {
    use leptos_use::{use_websocket, UseWebsocketReturn};
    let UseWebsocketReturn {
        ready_state,
        message,
        open,
        close,
        ..
    } = use_websocket(format!("ws://localhost:3e000/jam/users?jam_id={}", jam_id.get()).as_str());
    view! {
        <button on:click=move |_| close()>"Close"</button>
        <button on:click=move |_| open()>"Open"</button>
        "message:"{message}
        <br></br>
        "ready state:"{ready_state().to_string()}
    }
}
