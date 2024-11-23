use leptos::prelude::*;

#[component]
pub fn Modal(#[prop(into)] visible: Signal<bool>, children: Children) -> impl IntoView {
    view! {
        <div
            class="modal-wrapper"
            style:display=move || {
                match visible.get() {
                    true => "",
                    false => "none",
                }
            }
        >

            <dialog class="modal" prop:open=visible>
                {children()}
            </dialog>
        </div>
    }
}
