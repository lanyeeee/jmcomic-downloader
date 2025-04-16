use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ToggleFavoriteRespData {
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
