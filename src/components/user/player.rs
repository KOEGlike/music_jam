use crate::components::host::{millis_to_min_sec, set_bg_img};
use crate::general::types::Song;
use leptos::{
    logging::{error, log},
    prelude::*,
    *,
};

#[component]
pub fn Player(
    #[prop(into)] position: Signal<f32>,
    #[prop(into)] current_song: Signal<Option<Song>>,
) -> impl IntoView {
    create_effect(move |_| {
        current_song.with(|song| {
            if let Some(song) = song {
                set_bg_img(&song.image_url);
            }
        });
    });

    let song_length = move || current_song().map(|s| s.duration).unwrap_or_default();

    view! {
        <div class="player">
            <img
                prop:src=move || current_song().map(|s| s.image_url).unwrap_or_default()
                alt="the album cover of the current song"
            />

            <div class="info">
                <div class="title">
                    {move || current_song().map(|s| s.name).unwrap_or_default()}
                </div>
                <div class="artist">
                    {move || current_song().map(|s| s.artists.join(", ")).unwrap_or_default()}
                </div>
            </div>

            <div class="progress">
                <div class="bar">
                    <div
                        class="position"
                        style:width=move || format!("{}%", position() * 100.0)
                    ></div>
                </div>
                <div class="times">
                    <div>
                        {move || millis_to_min_sec((position() * song_length() as f32) as u32)}
                    </div>
                    <div>{move || millis_to_min_sec(song_length())}</div>
                </div>
            </div>
        </div>
    }
}
