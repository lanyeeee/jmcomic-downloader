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
pub struct UserProfileRespData {
    pub uid: String,
    pub username: String,
    pub email: String,
    pub emailverified: String,
    pub photo: String,
    pub fname: String,
    pub gender: String,
    pub message: Option<String>,
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
