pub mod app;
pub use crate::app::components;

#[cfg(feature = "ssr")]
pub mod router;
#[cfg(feature = "ssr")]
pub mod startup;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
