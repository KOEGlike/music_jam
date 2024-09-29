//pub mod fileserv;

use crate::{app::shell, model::AppState};
use axum::{routing::get, Router};

use leptos::prelude::*;
use leptos_axum::*;
use leptos_router::RouteListing;

use crate::app::App;

pub fn new(
    leptos_routes: Vec<AxumRouteListing>,
    app_state: AppState,
    leptos_options: LeptosOptions,
) -> Router {
    Router::new()
        .leptos_routes_with_context(
            &app_state,
            leptos_routes,
            {
                let state = app_state.clone();
                move || provide_context(state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .route("/socket", get(crate::socket::socket))
        .with_state(app_state.clone())
}
