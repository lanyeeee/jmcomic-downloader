use serde::{Deserialize, Serialize};
use specta::Type;

use super::SeriesRespData;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetChapterRespData {
    pub id: i64,
    pub series: Vec<SeriesRespData>,
    pub tags: String,
    pub name: String,
    pub images: Vec<String>,
    pub addtime: String,
    #[serde(rename = "series_id")]
    pub series_id: String,
    #[serde(rename = "is_favorite")]
    pub is_favorite: bool,
    pub liked: bool,
}
