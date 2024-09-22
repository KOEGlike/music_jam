use crate::components::user::CreateUser;
use leptos::prelude::*;
use leptos_router::{hooks::*, params::*, *};

#[component]
pub fn CreateUserPage() -> impl IntoView {
    #[derive(PartialEq, Params)]
    struct JamId {
        id: String,
    }
    let jam_id = use_params::<JamId>();
    let jam_id = move || {
        jam_id.with(|jam_id| {
            jam_id
                .as_ref()
                .map(|jam_id| jam_id.id.clone())
                .unwrap_or_else(|_| {
                    let navigate = use_navigate();
                    navigate("/", NavigateOptions::default());
                    warn!("No jam id provided, redirecting to home page");
                    "".to_string()
                })
        })
    };
    let jam_id = Signal::derive(jam_id);

    view! {
        <div class="create-user-page">
            <CreateUser jam_id=jam_id.get_untracked()/>
        </div>
    }
}
