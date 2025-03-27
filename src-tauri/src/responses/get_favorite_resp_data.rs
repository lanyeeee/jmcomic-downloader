use serde::{Deserialize, Serialize};
use specta::Type;

use super::{CategoryRespData, CategorySubRespData};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GetFavoriteRespData {
    pub list: Vec<ComicInFavoriteRespData>,
    #[serde(rename = "folder_list")]
    pub folder_list: Vec<FavoriteFolderRespData>,
    pub total: String,
    pub count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ComicInFavoriteRespData {
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
