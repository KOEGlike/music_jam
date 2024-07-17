use crate::app::general::*;
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn UsersBar(
    #[prop(into)]
    users: Signal<Option<Vec<User>>>,
    #[prop(optional)] kick_user: Option<Callback<String, ()>>,
) -> impl IntoView {
    let is_host = kick_user.is_some();

    view! {
        <div class="user-bar">
            {move || {
                if users.with(|users| users.is_none()) {
                    let mut vec = Vec::new();
                    for _ in 0..5 {
                        vec.push(view! {<div/>});
                    }
                    vec.into_view()
                } else {
                    ().into_view()
                }
            }}
            <For
                each=move || users().unwrap_or_default()
                key=|user| user.id.clone()
                children=move |user| {
                    view! {
                        <div
                            title=&user.name
                            class:kick-user=is_host
                            class="icon"
                            on:click={
                                let user_id = user.id.clone();
                                move |_| {
                                    if let Some(ref kick_user) = kick_user {
                                        kick_user(user_id.clone());
                                    }
                                }
                            }
                        >

                            <img
                                src=format!("/uploads/{}.webp", &user.id)
                                alt=format!("This is the profile picture of {}", &user.name)
                            />
                            <svg viewBox=IoClose.view_box inner_html=IoClose.data></svg>
                        </div>
                    }
                }
            />

        </div>
    }
}
