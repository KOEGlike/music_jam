use leptos::prelude::{ReadSignal, WriteSignal};

use super::{Song, User, Votes, real_time::SearchResult};

pub struct WsClientWrapper {
    pub search_result: ReadSignal<Option<SearchResult>>,
    set_search_result: WriteSignal<Option<SearchResult>>,
    pub songs: ReadSignal<Option<Vec<Song>>>,
    set_songs: WriteSignal<Option<Vec<Song>>>,
    pub users: ReadSignal<Option<Vec<User>>>,
    set_users: WriteSignal<Option<Vec<User>>>,
    pub votes: ReadSignal<Votes>,
    set_votes: WriteSignal<Votes>,
    pub position: ReadSignal<f32>,
    set_position: WriteSignal<f32>,
    pub current_song: ReadSignal<Option<Song>>,
    set_current_song: WriteSignal<Option<Song>>,
}
