use serde::{Deserialize, Serialize};
use specta::Type;

use super::{GetComicRespData, SearchRespData, SeriesRespData};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedirectRespData {
    #[serde(rename = "search_query")]
    pub search_query: String,
    pub total: i64,
    #[serde(rename = "redirect_aid")]
    pub redirect_aid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchResp {
    SearchRespData(SearchRespData),
    // 用Box包装GetComicRespData，因为GetComicRespData比SearchRespData大得多
    // 如果不用Box包装，即使SearchResp的类型是SearchRespData，也会占用与GetComicRespData一样大的内存
    ComicRespData(Box<GetComicRespData>),
}

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
