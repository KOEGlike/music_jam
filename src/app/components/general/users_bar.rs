use crate::app::general::*;
use icondata::IoClose;
use leptos::{logging::log, prelude::*, *};
use std::rc::Rc;

#[component]
pub fn UsersBar(
    users: ReadSignal<Vec<User>>,
    #[prop(optional)] kick_user: Option<Callback<String, ()>>,
) -> impl IntoView

{
    let is_host = kick_user.is_some();

    view! {
        <For
            each=users
            key=|user| user.id.clone()
            children=move |user| {
                view! {
                    <div
                        title=&user.name
                        class:kick-user=is_host
                        class="user-icon"
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
                            src=&user.id
                            alt=format!("This is the profile picture of {}", &user.name)
                        />
                        <svg viewBox=IoClose.view_box inner_html=IoClose.data></svg>
                    </div>
                }
            }
        />
    }
}
