use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    responses::string_to_i64,
    types::{Category, CategorySub},
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetWeeklyRespData {
    pub total: i64,
    pub list: Vec<ComicInWeeklyRespData>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ComicInWeeklyRespData {
    #[serde(deserialize_with = "string_to_i64")]
    pub id: i64,
    pub author: String,
    pub description: String,
    pub name: String,
    pub image: String,
    pub category: Category,
    pub category_sub: CategorySub,
    pub liked: bool,
    pub is_favorite: bool,
    pub update_at: i64,
}
