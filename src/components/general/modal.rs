use leptos::prelude::*;

#[component]
pub fn Modal(
    #[prop(into)]
    visible: Signal<bool>,
    children: Children
) -> impl IntoView {
    view! {
        <dialog class="modal" prop:open=visible>
            <div class="modal-content">{children()}</div>
        </dialog>
    }
}