use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::responses::{
    CategoryRespData, CategorySubRespData, ComicInSearchRespData, SearchResp, SearchRespData,
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
                let comic = Comic::from_comic_resp_data(app, *get_comic_resp);
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
        let content = search_resp_data
            .content
            .into_iter()
            .map(|comic| ComicInSearch::from_resp_data(app, comic))
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
}

impl ComicInSearch {
    pub fn from_resp_data(
        app: &AppHandle,
        resp_data: ComicInSearchRespData,
    ) -> anyhow::Result<ComicInSearch> {
        let id: i64 = resp_data.id.parse().context("将id解析为i64失败")?;
        let is_downloaded = Comic::get_is_downloaded(app, &resp_data.name);

        let comic = ComicInSearch {
            id,
            author: resp_data.author,
            name: resp_data.name,
            image: resp_data.image,
            category: resp_data.category,
            category_sub: resp_data.category_sub,
            liked: resp_data.liked,
            is_favorite: resp_data.is_favorite,
            update_at: resp_data.update_at,
            is_downloaded,
        };

        Ok(comic)
    }
}
