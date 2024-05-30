use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use crate::fileserv::file_and_error_handler;
use crate::{app::*, AppState};
use sqlx::PgPool;
use axum::Router;

pub async fn init () -> Result<(), Box<dyn std::error::Error>>{
   
    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await?;
    let leptos_options: LeptosOptions = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let state = AppState {
        db_pool: PgPool::connect(
            "postgresql://localhost:5432/jam_db?user=jammer",
        )
        .await?,
    };
    

    // build our application with a route
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(state.clone());
            },
            App,
        )
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await?;
    Ok(())
}