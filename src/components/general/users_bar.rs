use crate::model::*;
use icondata::IoClose;
use leptos::{either::Either, logging::log, prelude::*};
use std::rc::Rc;

#[component]
pub fn UsersBar(
    #[prop(into)] users: Signal<Option<Vec<User>>>,
    #[prop(optional)] kick_user: Option<Callback<String>>,
    close: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="bar">
            <button on:click=move |_| {
                close.run(());
            }>
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 31 30">
                    <path
                        d="M3.5 30L0.5 27L12.5 15L0.5 3L3.5 0L15.5 12L27.5 0L30.5 3L18.5 15L30.5 27L27.5 30L15.5 18L3.5 30Z"
                        fill="#EBF6E8"
                    ></path>
                </svg>
            </button>
            <div class="users">
                {move || {
                    if users.with(|users| users.is_none()) {
                        let mut vec = Vec::new();
                        for _ in 0..5 {
                            vec.push(view! { <div></div> });
                        }
                        Either::Left(vec.into_view())
                    } else {
                        Either::Right(())
                    }
                }}
                {move || {
                    if let Some(users) = users.get() {
                        if users.is_empty() {
                            return Either::Left(
                                view! { <div class="no-users">"No users in this jam 😔"</div> },
                            );
                        }
                    }
                    Either::Right(())
                }}
                <For
                    each=move || users.get().unwrap_or_default()
                    key=|user| user.id.clone()
                    children=move |user| {
                        let user_id = Rc::new(user.id);
                        view! {
                            <div title=user.name.clone() class="user">
                                <img
                                    src=format!("/uploads/{}.webp", user_id)
                                    alt=format!(
                                        "This is the profile picture of {}",
                                        user.name.clone(),
                                    )
                                />
                                {if let Some(kick_user) = kick_user {
                                    Either::Left(
                                        view! {
                                            <svg
                                                on:click={
                                                    let user_id = Rc::clone(&user_id);
                                                    move |_| {
                                                        log!("kicking user {}", user_id);
                                                        kick_user.run((*user_id).clone());
                                                    }
                                                }

                                                viewBox=IoClose.view_box
                                                inner_html=IoClose.data
                                            ></svg>
                                        },
                                    )
                                } else {
                                    Either::Right(())
                                }}

                            </div>
                        }
                    }
                />

            </div>
        </div>
    }
}
