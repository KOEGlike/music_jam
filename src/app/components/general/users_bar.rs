use crate::app::general::*;
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};

#[component]
pub fn UsersBar(
    #[prop(into)] users: Signal<Option<Vec<User>>>,
    #[prop(optional)] kick_user: Option<Callback<String>>,
) -> impl IntoView {

    view! {
        <div class="user-bar">
            {move || {
                if users.with(|users| users.is_none()) {
                    let mut vec = Vec::new();
                    for _ in 0..5 {
                        vec.push(view! { <div></div> });
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
                        <div title=&user.name class="user">

                            <img
                                src=format!("/uploads/{}.webp", &user.id)
                                alt=format!("This is the profile picture of {}", &user.name)
                            />

                            {if let Some(kick_user) = kick_user {
                                view! {
                                    <svg
                                        on:click=move|_|{
                                            log!("kicking user {}", user.id);
                                            kick_user(user.id.clone());
                                        }
                                        viewBox=IoClose.view_box
                                        inner_html=IoClose.data
                                    ></svg>
                                }
                                    .into_view()
                            } else {
                                ().into_view()
                            }}

                        </div>
                    }
                }
            />

        </div>
    }
}
