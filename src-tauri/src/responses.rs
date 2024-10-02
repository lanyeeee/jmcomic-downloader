use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JmResp {
    pub code: i64,
    pub data: serde_json::Value,
    #[serde(default)]
    pub error_msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileRespData {
    pub uid: String,
    pub username: String,
    pub email: String,
    pub emailverified: String,
    pub photo: String,
    pub fname: String,
    pub gender: String,
    pub message: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub coin: i64,
    #[serde(rename = "album_favorites")]
    pub album_favorites: i64,
    pub s: String,
    #[serde(rename = "level_name")]
    pub level_name: String,
    pub level: i64,
    pub next_level_exp: i64,
    pub exp: String,
    pub exp_percent: f64,
    #[serde(rename = "album_favorites_max")]
    pub album_favorites_max: i64,
    #[serde(rename = "ad_free")]
    pub ad_free: bool,
    pub charge: String,
    pub jar: String,
    #[serde(rename = "invitation_qrcode")]
    pub invitation_qrcode: String,
    #[serde(rename = "invitation_url")]
    pub invitation_url: String,
    #[serde(rename = "invited_cnt")]
    pub invited_cnt: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchRespData {
    #[serde(rename = "search_query")]
    pub search_query: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub total: i64,
    pub content: Vec<AlbumInSearchRespData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInSearchRespData {
    pub id: String,
    pub author: String,
    pub name: String,
    pub image: String,
    pub category: CategoryRespData,
    #[serde(rename = "category_sub")]
    pub category_sub: CategorySubRespData,
    pub liked: bool,
    #[serde(rename = "is_favorite")]
    pub is_favorite: bool,
    #[serde(rename = "update_at")]
    pub update_at: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRespData {
    pub id: String,
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategorySubRespData {
    pub id: Option<String>,
    pub title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AlbumRespData {
    pub id: i64,
    pub name: String,
    pub addtime: String,
    pub description: String,
    #[serde(rename = "total_views")]
    pub total_views: String,
    pub likes: String,
    pub series: Vec<SeriesRespData>,
    #[serde(rename = "series_id")]
    pub series_id: String,
    #[serde(rename = "comment_total")]
    pub comment_total: String,
    pub author: Vec<String>,
    pub tags: Vec<String>,
    pub works: Vec<String>,
    pub actors: Vec<String>,
    #[serde(rename = "related_list")]
    pub related_list: Vec<RelatedListRespData>,
    pub liked: bool,
    #[serde(rename = "is_favorite")]
    pub is_favorite: bool,
    #[serde(rename = "is_aids")]
    pub is_aids: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SeriesRespData {
    pub id: String,
    pub name: String,
    pub sort: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RelatedListRespData {
    pub id: String,
    pub author: String,
    pub name: String,
    pub image: String,
}

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
    // 用Box包装AlbumRespData，因为Album比SearchRespData大得多
    // 如果不用Box包装，即使SearchResp的类型是SearchRespData，也会占用与AlbumRespData一样大的内存
    AlbumRespData(Box<AlbumRespData>),
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChapterRespData {
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteRespData {
    pub list: Vec<AlbumInFavoriteRespData>,
    #[serde(rename = "folder_list")]
    pub folder_list: Vec<FavoriteFolderRespData>,
    pub total: String,
    pub count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInFavoriteRespData {
    pub id: String,
    pub author: String,
    pub description: Option<String>,
    pub name: String,
    #[serde(rename = "latest_ep")]
    pub latest_ep: Option<String>,
    #[serde(rename = "latest_ep_aid")]
    pub latest_ep_aid: Option<String>,
    pub image: String,
    pub category: CategoryRespData,
    #[serde(rename = "category_sub")]
    pub category_sub: CategorySubRespData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteFolderRespData {
    #[serde(rename = "FID")]
    pub fid: String,
    #[serde(rename = "UID")]
    pub uid: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ToggleFavoriteResp {
    pub status: String,
    pub msg: String,
    #[serde(rename = "type")]
    pub toggle_type: ToggleType,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ToggleType {
    #[default]
    Add,
    Remove,
}
