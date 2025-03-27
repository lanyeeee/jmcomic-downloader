use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::responses::{SearchResp, SearchRespData};

use super::Comic;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchResult {
    SearchRespData(SearchRespData),
    // 用Box包装Comic，因为Comic比SearchRespData大得多
    // 如果不用Box包装，即使SearchResult的类型是SearchRespData，也会占用与Comic一样大的内存
    Comic(Box<Comic>),
}

impl SearchResult {
    pub fn from_search_resp(app: &AppHandle, search_resp: SearchResp) -> Self {
        match search_resp {
            SearchResp::SearchRespData(search_resp) => SearchResult::SearchRespData(search_resp),
            SearchResp::ComicRespData(get_comic_resp) => {
                let comic = Comic::from_comic_resp_data(app, *get_comic_resp);
                SearchResult::Comic(Box::new(comic))
            }
        }
    }
}
