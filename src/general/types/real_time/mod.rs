mod changed;
pub use changed::*;

mod update;
pub use update::*;

mod request;
pub use request::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Message{
    Request(Request),
    Update(Update),
}

#[cfg(feature = "ssr")]
mod channel_update;
#[cfg(feature = "ssr")]
pub use channel_update::*;
