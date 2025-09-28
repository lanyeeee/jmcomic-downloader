use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetWeeklyInfoRespData {
    pub categories: Vec<CategoryInWeeklyInfo>,
    #[serde(rename = "type")]
    pub type_field: Vec<WeeklyType>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CategoryInWeeklyInfo {
    pub id: String,
    pub title: String,
    pub time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct WeeklyType {
    pub id: String,
    pub title: String,
}
