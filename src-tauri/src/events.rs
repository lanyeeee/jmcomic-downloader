use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

pub mod prelude {
    pub use crate::events::{
        DownloadChapterEndEvent, DownloadChapterPendingEvent, DownloadChapterStartEvent,
        DownloadImageErrorEvent, DownloadImageSuccessEvent, DownloadSpeedEvent,
        UpdateOverallDownloadProgressEvent,
    };
}

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadChapterPendingEventPayload {
    pub chapter_id: i64,
    pub title: String,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct DownloadChapterPendingEvent(pub DownloadChapterPendingEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadChapterStartEventPayload {
    pub chapter_id: i64,
    pub title: String,
    pub total: u32,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct DownloadChapterStartEvent(pub DownloadChapterStartEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadImageSuccessEventPayload {
    pub chapter_id: i64,
    pub url: String,
    pub downloaded_count: u32,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct DownloadImageSuccessEvent(pub DownloadImageSuccessEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadImageErrorEventPayload {
    pub chapter_id: i64,
    pub url: String,
    pub err_msg: String,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct DownloadImageErrorEvent(pub DownloadImageErrorEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadChapterEndEventPayload {
    pub chapter_id: i64,
    pub err_msg: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct DownloadChapterEndEvent(pub DownloadChapterEndEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOverallDownloadProgressEventPayload {
    pub downloaded_image_count: u32,
    pub total_image_count: u32,
    pub percentage: f64,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct UpdateOverallDownloadProgressEvent(pub UpdateOverallDownloadProgressEventPayload);

#[derive(Serialize, Deserialize, Clone, Type)]
#[serde(rename_all = "camelCase")]
pub struct DownloadSpeedEventPayload {
    pub speed: String,
}
#[derive(Serialize, Deserialize, Clone, Type, Event)]
pub struct DownloadSpeedEvent(pub DownloadSpeedEventPayload);
