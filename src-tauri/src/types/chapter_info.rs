use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::utils::filename_filter;

use super::Comic;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChapterInfo {
    pub chapter_id: i64,
    pub chapter_title: String,
    pub comic_id: i64,
    pub comic_title: String,
    pub author: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_downloaded: Option<bool>,
    pub order: i64,
}

impl ChapterInfo {
    pub fn get_is_downloaded(app: &AppHandle, comic_title: &str, chapter_title: &str) -> bool {
        let comic_download_dir = Comic::get_comic_download_dir(app, comic_title);

        let chapter_title = filename_filter(chapter_title);
        comic_download_dir.join(chapter_title).exists()
    }

    pub fn get_temp_download_dir(&self, app: &AppHandle) -> PathBuf {
        let comic_download_dir = Comic::get_comic_download_dir(app, &self.comic_title);

        let chapter_title = filename_filter(&self.chapter_title);
        comic_download_dir.join(format!(".下载中-{chapter_title}")) // 以 `.下载中-` 开头，表示是临时目录
    }

    pub fn get_chapter_download_dir(&self, app: &AppHandle) -> PathBuf {
        let comic_download_dir = Comic::get_comic_download_dir(app, &self.comic_title);

        let chapter_title = filename_filter(&self.chapter_title);
        comic_download_dir.join(chapter_title)
    }
}
