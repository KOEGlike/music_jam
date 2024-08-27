use crate::components::host::millis_to_min_sec;
use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};

use crate::general::types::Song;

#[component]
pub fn Player(
    #[prop(into)]
    position: Signal<f32>, 
    #[prop(into)]
    current_song: Signal<Option<Song>>
) -> impl IntoView {
    let song_length = move || current_song().map(|s| s.duration).unwrap_or_default();
    view! {
        <div class="player">
            <img
                prop:src=current_song().map(|s| s.image_url).unwrap_or_default()
                alt="the album cover of the current song"
            />

            <div class="info">
                <div class="title">{current_song().map(|s| s.name).unwrap_or_default()}</div>
                <div class="artist">
                    {current_song().map(|s| s.artists.join(", ")).unwrap_or_default()}
                </div>
            </div>

            <div class="progress">
                <div class="bar">
                    <div
                        class="position"
                        style:width=move || {
                            let song_length = song_length();
                            let percentage = if song_length == 0 {
                                0.0
                            } else {
                                (position() / song_length as f32) * 100.0
                            };
                            format!("{}%", percentage)
                        }
                    >
                    </div>
                </div>
                <div class="times">
                    <div>
                        {move || { millis_to_min_sec((position() * song_length() as f32) as u32) }}
                    </div>
                    <div>{move || millis_to_min_sec(song_length())}</div>
                </div>
            </div>
        </div>
    }
}
