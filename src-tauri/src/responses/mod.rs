mod get_chapter_resp_data;
mod get_comic_resp_data;
mod get_favorite_resp_data;
mod get_user_profile_resp_data;
mod get_weekly_info_resp_data;
mod get_weekly_resp_data;
mod search_resp;
mod toggle_favorite_resp_data;

pub use get_chapter_resp_data::*;
pub use get_comic_resp_data::*;
pub use get_favorite_resp_data::*;
pub use get_user_profile_resp_data::*;
pub use get_weekly_info_resp_data::*;
pub use get_weekly_resp_data::*;
pub use search_resp::*;
pub use toggle_favorite_resp_data::*;

use serde::{Deserialize, Serialize};
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
pub struct SeriesRespData {
    pub id: String,
    pub name: String,
    pub sort: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRespData {
    pub id: Option<String>,
    pub title: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CategorySubRespData {
    pub id: Option<String>,
    pub title: Option<String>,
}

pub fn string_to_i64<'de, D>(d: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    let value: Value = serde::Deserialize::deserialize(d)?;

    match value {
        #[allow(clippy::cast_possible_truncation)]
        Value::Number(n) => Ok(n.as_i64().unwrap_or(0)),
        Value::String(s) => Ok(s.parse().unwrap_or(0)),
        _ => Err(serde::de::Error::custom(
            "`string_to_i64` 失败，value类型不是 `Number` 或 `String`",
        )),
    }
}
