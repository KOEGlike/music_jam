use codee::binary::MsgpackSerdeCodec;
use leptos::{logging::*, prelude::*};
use leptos_use::{UseWebSocketReturn, core::ConnectionReadyState, use_websocket};
use std::marker::PhantomData;

use super::{
    Song, User, Votes,
    real_time::{self, search},
};

pub trait Role: 'static {}
#[derive(Debug, Clone, Copy)]
pub struct HostRole;
impl Role for HostRole {}
#[derive(Debug, Clone, Copy)]
pub struct UserRole;
impl Role for UserRole {}

pub struct WsClientWrapper<R: Role> {
    pub search_result: Signal<Option<real_time::SearchResult>>,
    set_search_result: WriteSignal<Option<real_time::SearchResult>>,
    pub songs: Signal<Option<Vec<Song>>>,
    set_songs: WriteSignal<Option<Vec<Song>>>,
    pub users: Signal<Option<Vec<User>>>,
    set_users: WriteSignal<Option<Vec<User>>>,
    pub votes: Signal<Votes>,
    set_votes: WriteSignal<Votes>,
    pub position: Signal<f32>,
    set_position: WriteSignal<f32>,
    pub current_song: Signal<Option<Song>>,
    set_current_song: WriteSignal<Option<Song>>,
    pub errors: Signal<Vec<super::Error>>,
    set_errors: WriteSignal<Vec<super::Error>>,
    pub ended: Signal<bool>,
    set_ended: WriteSignal<bool>,

    initial_update: Signal<Option<Result<real_time::Update, super::Error>>>,

    pub ready_state: Signal<ConnectionReadyState>,

    send: Box<dyn Fn(&real_time::Request) + Send + Sync>,
    message: Signal<Option<real_time::Update>>,
    pub close: Box<dyn Fn() + Send + Sync>,

    id: String,

    _role: PhantomData<R>,
}

impl<R: Role> WsClientWrapper<R> {
    pub fn new(
        id: String,
        initial_update: Signal<Option<Result<real_time::Update, super::Error>>>,
    ) -> Self {
        let UseWebSocketReturn {
            ready_state,
            message,
            close,
            send,
            ..
        } = use_websocket::<real_time::Request, real_time::Update, MsgpackSerdeCodec>(&format!(
            "/socket?id={}",
            id
        ));

        let (search_result, set_search_result) = signal(None);
        let (songs, set_songs) = signal(None);
        let (users, set_users) = signal(None);
        let (votes, set_votes) = signal(Votes::default());
        let (position, set_position) = signal(0.0);
        let (current_song, set_current_song) = signal(None);
        let (errors, set_errors) = signal(Vec::new());
        let (ended, set_ended) = signal(false);

        {
            let close = close.clone();
            Effect::new(move |_| {
                if let Some(update) = message().or_else(move || match initial_update() {
                    Some(Ok(update)) => Some(update),
                    Some(Err(e)) => {
                        warn!("Error getting initial update: {:#?}", e);
                        None
                    }
                    None => None,
                }) {
                    if let Some(users) = update.users {
                        set_users.set(Some(users));
                    }
                    if let Some(songs) = update.songs {
                        set_songs.set(Some(songs));
                    }
                    if let Some(votes) = update.votes {
                        set_votes.set(votes);
                    }
                    set_errors.set(update.errors);
                    if update.ended.is_some() {
                        close();
                        set_ended.set(true);
                    }
                    if let Some(search_result) = update.search {
                        set_search_result.set(Some(search_result));
                    }
                    if let Some(position) = update.position {
                        set_position.set(position);
                    }
                    if let Some(current_song) = update.current_song {
                        set_current_song.set(current_song);
                    }
                }
            });
        }

        Self {
            search_result: search_result.into(),
            set_search_result,
            songs: songs.into(),
            set_songs,
            users: users.into(),
            set_users,
            votes: votes.into(),
            set_votes,
            position: position.into(),
            set_position,
            current_song: current_song.into(),
            set_current_song,
            errors: errors.into(),
            set_errors,
            ended: ended.into(),
            set_ended,

            initial_update,

            ready_state,

            send: Box::new(send),
            message,
            close: Box::new(close),

            id,

            _role: PhantomData,
        }
    }

    pub fn remove_song(&self, song_id: String) {
        let request = real_time::Request::RemoveSong { song_id };
        (self.send)(&request);
    }
}

impl WsClientWrapper<UserRole> {
    pub fn search(&self, query: String, search_id: String) {
        let request = real_time::Request::Search {
            query,
            id: search_id,
        };
        (self.send)(&request);
    }

    pub fn add_song(&self, song_id: String) {
        let request = real_time::Request::AddSong { song_id };
        (self.send)(&request);
    }

    pub fn add_vote(&self, song_id: String) {
        let request = real_time::Request::AddVote { song_id };
        (self.send)(&request);
    }

    pub fn remove_vote(&self, song_id: String) {
        let request = real_time::Request::RemoveVote { song_id };
        (self.send)(&request);
    }

    pub fn leave(&self) {
        let request = real_time::Request::KickUser {
            user_id: self.id.clone(),
        };
        (self.send)(&request);
    }
}

impl WsClientWrapper<HostRole> {
    pub fn kick_user(&self, user_id: String) {
        let request = real_time::Request::KickUser { user_id };
        (self.send)(&request);
    }
    pub fn set_song_position(&self, percentage: f32) {
        let request = real_time::Request::Position { percentage };
        (self.send)(&request);
    }
}
