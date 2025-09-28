use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::{
    responses::{string_to_i64, ComicInWeeklyRespData, GetWeeklyRespData},
    types::{Category, CategorySub},
    utils,
};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GetWeeklyResult {
    pub total: i64,
    pub list: Vec<ComicInWeekly>,
}

impl GetWeeklyResult {
    pub fn from_resp_data(app: &AppHandle, resp_data: GetWeeklyRespData) -> anyhow::Result<Self> {
        let id_to_dir_map =
            utils::create_id_to_dir_map(app).context("创建漫画ID到下载目录映射失败")?;

        let list = resp_data
            .list
            .into_iter()
            .map(|comic| ComicInWeekly::from_resp_data(comic, &id_to_dir_map))
            .collect();

        let get_weekly_result = GetWeeklyResult {
            total: resp_data.total,
            list,
        };

        Ok(get_weekly_result)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct ComicInWeekly {
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
    pub is_downloaded: bool,
    pub comic_download_dir: PathBuf,
}

impl ComicInWeekly {
    pub fn from_resp_data(
        resp_data: ComicInWeeklyRespData,
        id_to_dir_map: &HashMap<i64, PathBuf>,
    ) -> ComicInWeekly {
        let mut comic = ComicInWeekly {
            id: resp_data.id,
            author: resp_data.author,
            description: resp_data.description,
            name: resp_data.name,
            image: resp_data.image,
            category: resp_data.category,
            category_sub: resp_data.category_sub,
            liked: resp_data.liked,
            is_favorite: resp_data.is_favorite,
            update_at: resp_data.update_at,
            is_downloaded: false,
            comic_download_dir: PathBuf::new(),
        };

        comic.update_fields(id_to_dir_map);

        comic
    }

    pub fn update_fields(&mut self, id_to_dir_map: &HashMap<i64, PathBuf>) {
        if let Some(comic_download_dir) = id_to_dir_map.get(&self.id) {
            self.comic_download_dir = comic_download_dir.clone();
            self.is_downloaded = true;
        }
    }
}
