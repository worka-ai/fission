use crate::{Action, ActionId};
use fission_ir::WidgetNodeId;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoPlay {
    pub target: WidgetNodeId,
}

impl Action for VideoPlay {
    fn static_id() -> ActionId {
        *VIDEO_PLAY_ID
    }
}

lazy_static! {
    pub static ref VIDEO_PLAY_ID: ActionId = ActionId::from_name("fission_core::VideoPlay");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoPause {
    pub target: WidgetNodeId,
}

impl Action for VideoPause {
    fn static_id() -> ActionId {
        *VIDEO_PAUSE_ID
    }
}

lazy_static! {
    pub static ref VIDEO_PAUSE_ID: ActionId = ActionId::from_name("fission_core::VideoPause");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoStop {
    pub target: WidgetNodeId,
}

impl Action for VideoStop {
    fn static_id() -> ActionId {
        *VIDEO_STOP_ID
    }
}

lazy_static! {
    pub static ref VIDEO_STOP_ID: ActionId = ActionId::from_name("fission_core::VideoStop");
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSeek {
    pub target: WidgetNodeId,
    pub position_ms: u64,
}

impl Action for VideoSeek {
    fn static_id() -> ActionId {
        *VIDEO_SEEK_ID
    }
}

lazy_static! {
    pub static ref VIDEO_SEEK_ID: ActionId = ActionId::from_name("fission_core::VideoSeek");
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSetRate {
    pub target: WidgetNodeId,
    pub rate: f32,
}

impl Action for VideoSetRate {
    fn static_id() -> ActionId {
        *VIDEO_SET_RATE_ID
    }
}

lazy_static! {
    pub static ref VIDEO_SET_RATE_ID: ActionId = ActionId::from_name("fission_core::VideoSetRate");
}
