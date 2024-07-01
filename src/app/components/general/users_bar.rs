use crate::app::general::*;
use leptos::{logging::log, prelude::*, *};
use icondata::IoClose;

#[component]
pub fn UsersBar(
    users: ReadSignal<Vec<User>>,
    #[prop(optional)]
    mut kick_user: Option<impl FnMut(&str)+'static>,
) -> impl IntoView {
    let is_host=kick_user.is_some();
    
    view! {
        <For
            each=users
            key=|user| user.id.clone()
            children=move |user| {
                let user_name = &user.name;
                let user_pfp = &user.pfp_id;
                let user_id = user.id.clone();
                
                let on_click = move |_| {
                    if let Some(kick_user) = kick_user {
                        kick_user(&user_id);
                    }
                };
                view! {
                    <div 
                        title=user_name 
                        class:kick-user=is_host
                        class="user-icon" 
                        on:click=on_click
                    >
                        <img src=user_pfp alt={format!("This is the profile picture of {}", user_name)}/>
                        <svg viewBox=IoClose.view_box inner_html=IoClose.data/>
                    </div>
                }
            }
        />
    }
}
