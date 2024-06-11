use crate::fileserv::file_and_error_handler;
use crate::{app::*, general_types::AppState};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};

use axum::Router;

pub async fn init() -> Result<(), Box<dyn std::error::Error>> {
    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    dotenvy::dotenv()?;

    let conf = get_configuration(None).await?;
    let leptos_options: LeptosOptions = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let state = AppState::new(leptos_options).await?;

    // build our application with a route
    let app = Router::new()
        .leptos_routes_with_context(
            &state,
            routes,
             {
                let state = state.clone();
                move ||provide_context(state.clone())
            },
            App,
        )
        .fallback(file_and_error_handler)
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
