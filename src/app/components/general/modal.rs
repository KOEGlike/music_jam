use leptos::*;

#[component]
pub fn Modal(
    visible: ReadSignal<bool>,
    children: Children
) -> impl IntoView {
    view! {
       <div id="centerpoint">
            <dialog class="dialog" prop:open=visible>
                {children()}
            </dialog>
        </div>
    }
}