use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

pub mod prelude {
    pub use crate::events::{DownloadEvent, SetProxyEvent};
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(tag = "event", content = "data")]
pub enum DownloadEvent {
    #[serde(rename_all = "camelCase")]
    ChapterPending {
        chapter_id: i64,
        album_title: String,
        chapter_title: String,
    },

    #[serde(rename_all = "camelCase")]
    ChapterStart { chapter_id: i64, total: u32 },

    #[serde(rename_all = "camelCase")]
    ChapterEnd {
        chapter_id: i64,
        err_msg: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    ImageSuccess {
        chapter_id: i64,
        url: String,
        current: u32,
    },

    #[serde(rename_all = "camelCase")]
    ImageError {
        chapter_id: i64,
        url: String,
        err_msg: String,
    },

    #[serde(rename_all = "camelCase")]
    OverallUpdate {
        downloaded_image_count: u32,
        total_image_count: u32,
        percentage: f64,
    },

    #[serde(rename_all = "camelCase")]
    OverallSpeed { speed: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(tag = "event", content = "data")]
pub enum SetProxyEvent {
    #[serde(rename_all = "camelCase")]
    Error { err_msg: String },
}
