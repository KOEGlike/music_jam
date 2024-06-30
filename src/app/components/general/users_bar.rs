use crate::app::general::*;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn UsersBar(
    users: ReadSignal<Vec<User>>,
    is_host: bool,
    kick_user: impl Fn(&str),
) -> impl IntoView {
    view! {
        <For
            each=users
            key=|user| user.id.clone()
            children=|user| {
                view! {
                    <div>
                        <span>{user.name.clone()}</span>

                    </div>
                }
            }
        />
    }
}
