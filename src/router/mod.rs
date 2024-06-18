pub mod fileserv;
pub use fileserv::*;

use crate::general_types::AppState;
use axum::{Router, routing::get};

use leptos_router::RouteListing;
use leptos::*;
use leptos_axum::*;

use crate::app::App;

pub fn new(leptos_routes:Vec<RouteListing>, app_state:AppState) -> Router {
    Router::new()
        .leptos_routes_with_context(
            &app_state,
            leptos_routes,
             {
                let state = app_state.clone();
                move ||provide_context(state.clone())
            },
            App,
        )
        .route("/jam/users", get(crate::app::components::general::users_bar::get_users_handler))
        .fallback(file_and_error_handler)
        .with_state(app_state.clone())
}