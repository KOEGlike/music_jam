pub mod fileserv;
pub use fileserv::*;

use crate::app::general::AppState;
use axum::{routing::get, Router};

use leptos::*;
use leptos_axum::*;
use leptos_router::RouteListing;

use crate::app::App;

pub fn new(leptos_routes: Vec<RouteListing>, app_state: AppState) -> Router {
    Router::new()
        .leptos_routes_with_context(
            &app_state,
            leptos_routes,
            {
                let state = app_state.clone();
                move || provide_context(state.clone())
            },
            App,
        )
        .route("/socket", get(crate::app::socket::socket))
        .fallback(file_and_error_handler)
        .with_state(app_state.clone())
}

