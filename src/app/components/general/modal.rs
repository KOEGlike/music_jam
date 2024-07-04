use leptos::*;

#[component]
pub fn Modal(
    visible: ReadSignal<bool>,
    children: Children
) -> impl IntoView {
    view! {
        <dialog class="modal" prop:open=visible>
            <div class="modal-content">{children()}</div>
        </dialog>
    }
}