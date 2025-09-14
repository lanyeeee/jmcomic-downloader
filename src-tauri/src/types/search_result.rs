use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::{
    responses::{
        CategoryRespData, CategorySubRespData, ComicInSearchRespData, SearchResp, SearchRespData,
    },
    utils,
};

use super::Comic;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchResultVariant {
    SearchResult(SearchResult),
    // 用Box包装Comic，因为Comic比SearchResult大得多
    // 如果不用Box包装，即使SearchResult的类型是SearchResult，也会占用与Comic一样大的内存
    Comic(Box<Comic>),
}

impl SearchResultVariant {
    pub fn from_search_resp(
        app: &AppHandle,
        search_resp: SearchResp,
    ) -> anyhow::Result<SearchResultVariant> {
        match search_resp {
            SearchResp::SearchRespData(search_resp_data) => {
                let search_result = SearchResult::from_resp_data(app, search_resp_data)?;
                Ok(SearchResultVariant::SearchResult(search_result))
            }
            SearchResp::ComicRespData(get_comic_resp) => {
                let comic = Comic::from_comic_resp_data(app, *get_comic_resp)?;
                Ok(SearchResultVariant::Comic(Box::new(comic)))
            }
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub search_query: String,
    pub total: i64,
    pub content: Vec<ComicInSearch>,
}

impl SearchResult {
    pub fn from_resp_data(
        app: &AppHandle,
        search_resp_data: SearchRespData,
    ) -> anyhow::Result<SearchResult> {
        let id_to_dir_map =
            utils::create_id_to_dir_map(app).context("创建漫画ID到下载目录映射失败")?;

        let content = search_resp_data
            .content
            .into_iter()
            .map(|comic| ComicInSearch::from_resp_data(comic, &id_to_dir_map))
            .collect::<anyhow::Result<_>>()?;

        let search_result = SearchResult {
            search_query: search_resp_data.search_query,
            total: search_resp_data.total,
            content,
        };

        Ok(search_result)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ComicInSearch {
    pub id: i64,
    pub author: String,
    pub name: String,
    pub image: String,
    pub category: CategoryRespData,
    pub category_sub: CategorySubRespData,
    pub liked: bool,
    pub is_favorite: bool,
    pub update_at: i64,
    pub is_downloaded: bool,
    pub comic_download_dir: PathBuf,
}

impl ComicInSearch {
    pub fn from_resp_data(
        resp_data: ComicInSearchRespData,
        id_to_dir_map: &HashMap<i64, PathBuf>,
    ) -> anyhow::Result<ComicInSearch> {
        let id: i64 = resp_data.id.parse().context("将id解析为i64失败")?;

        let mut comic = ComicInSearch {
            id,
            author: resp_data.author,
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

        Ok(comic)
    }

    pub fn update_fields(&mut self, id_to_dir_map: &HashMap<i64, PathBuf>) {
        if let Some(comic_download_dir) = id_to_dir_map.get(&self.id) {
            self.comic_download_dir = comic_download_dir.clone();
            self.is_downloaded = true;
        }
    }
}
