use serde::{Deserialize, Serialize};
use specta::Type;

use super::{string_to_i64, CategoryRespData, CategorySubRespData};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchRespData {
    #[serde(rename = "search_query")]
    pub search_query: String,
    #[serde(deserialize_with = "string_to_i64")]
    pub total: i64,
    pub content: Vec<ComicInSearchRespData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ComicInSearchRespData {
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
