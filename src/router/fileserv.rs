use axum::{
    body::Body,
    extract::State,
    response::IntoResponse,
    http::{Request, Response, StatusCode, Uri},
};
use axum::response::Response as AxumResponse;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use crate::model::AppState;
use crate::app::App;

pub async fn file_and_error_handler(uri: Uri, State(state): State<AppState>, req: Request<Body>) -> AxumResponse {
    let options=state.leptos_options;
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
        handler(req).await.into_response()
    }
}

async fn get_static_file(
    uri: Uri,
    root: &str,
) -> Result<Response<Body>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    // This path is relative to the cargo root
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.into_response()),
    }
}
