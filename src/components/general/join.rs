use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};

#[component]
pub fn JoinIsland() -> impl IntoView {
    let (jam_code, set_jam_code) = signal(String::from(""));
    let on_click = move |_| {
        if !jam_code.with(String::is_empty) {
            let navigate = use_navigate();
            navigate(
                format!("/create-user/{}", jam_code.get_untracked()).as_str(),
                NavigateOptions::default(),
            );
        }
    };

    view! {
        <div id="join-island">
            <div class="input-with-label">
                <label for="join-text-input">"Jam Code"</label>
                <input
                    type="text"
                    maxlength=6
                    prop:value=jam_code
                    on:input=move |ev| set_jam_code(event_target_value(&ev))
                    placeholder="ex. 786908"
                    class="text-input"
                    id="join-text-input"
                    class:glass-element-err=move || jam_code.with(String::is_empty)
                />
            </div>
            <button on:click=on_click class="button">
                "Join"
            </button>
        </div>
    }
}
