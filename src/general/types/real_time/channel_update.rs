use serde::{Serialize, Deserialize};
use crate::general::types::*;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelUpdate{
    pub update: real_time::Update,
    pub changed: real_time::Changed,
}