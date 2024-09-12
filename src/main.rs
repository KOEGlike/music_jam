#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use leptos::*;
    use leptos_axum::generate_route_list;
    use music_jam::router;
    use music_jam::{app::*, model::types::AppState};
println!("Starting server...");
    if dotenvy::dotenv().is_err(){
        eprintln!("didn't find env file")
    };
    println!("Loading configuration...");
    let conf = get_configuration(None).await.unwrap();
    println!("Configuration loaded...");
    let leptos_options: LeptosOptions = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);
    println!("Starting server on: {}", addr);
    println!("Loading state...");
    let state = AppState::new(leptos_options).await.unwrap();
    println!("State loaded...");
    println!("creating router...");
    // build our application with a route
    let app = router::new(routes, state);

    println!("creating listener...");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
   println!("listening on http://{}", &addr);
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
