use crate::router;
use crate::app::{*,general::types::AppState};
use leptos::*;
use leptos_axum::generate_route_list;

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
    let app = router::new(routes, state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
