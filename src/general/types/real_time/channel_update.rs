use std::vec::Vec;

use serde::{Serialize, Deserialize};
use crate::general::types::*;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelUpdate{
    pub errors: Vec<Error>,
    pub changed: real_time::Changed,
}