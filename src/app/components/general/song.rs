use crate::app::general::*;
use leptos::*;

#[component]
pub fn Song(
    song: Song,
    on_click: impl Fn() + 'static,
    #[prop(default = "".to_string())]
    #[prop(into)]
    class: String,
    children: Children,
) -> impl IntoView {
    view! {
        <div on:click=move |_| on_click() class={class+" song"}>
            <div>
                <img src=&song.image.url alt={format!("This is the album cover of {}", &song.name)}/>
                <div>
                    {&song.name}
                    <div>{&song.artists.join(", ")}"·"{song.duration%60}"."{song.duration/60}</div>
                </div>
            </div>
            {children()}
        </div>
    }
}
