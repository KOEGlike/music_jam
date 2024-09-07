#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use leptos::*;
    use leptos_axum::generate_route_list;
    use music_jam::router;
    use music_jam::{app::*, model::types::AppState};

    dotenvy::dotenv().unwrap();
    let conf = get_configuration(None).await.unwrap();
    let leptos_options: LeptosOptions = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    let state = AppState::new(leptos_options).await.unwrap();

    // build our application with a route
    let app = router::new(routes, state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
