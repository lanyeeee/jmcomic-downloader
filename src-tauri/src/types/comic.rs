use std::path::PathBuf;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::{
    config::Config,
    responses::{GetComicRespData, RelatedListRespData},
    utils::{self, filename_filter},
};

use super::ChapterInfo;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Comic {
    pub id: i64,
    pub name: String,
    pub addtime: String,
    pub description: String,
    #[serde(rename = "total_views")]
    pub total_views: String,
    pub likes: String,
    pub chapter_infos: Vec<ChapterInfo>,
    #[serde(rename = "series_id")]
    pub series_id: String,
    #[serde(rename = "comment_total")]
    pub comment_total: String,
    pub author: Vec<String>,
    pub tags: Vec<String>,
    pub works: Vec<String>,
    pub actors: Vec<String>,
    #[serde(rename = "related_list")]
    pub related_list: Vec<RelatedListRespData>,
    pub liked: bool,
    #[serde(rename = "is_favorite")]
    pub is_favorite: bool,
    #[serde(rename = "is_aids")]
    pub is_aids: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_downloaded: Option<bool>,
}

impl Comic {
    pub fn from_comic_resp_data(app: &AppHandle, comic: GetComicRespData) -> Self {
        let comic_title = utils::filename_filter(&comic.name);
        let mut chapter_infos: Vec<ChapterInfo> = comic
            .series
            .into_iter()
            .filter_map(|s| {
                let chapter_id = s.id.parse().ok()?;
                let order = s.sort.parse().ok()?;
                let mut chapter_title = format!("第{order}话");
                if !s.name.is_empty() {
                    chapter_title.push_str(&format!(" {}", utils::filename_filter(&s.name)));
                }
                let is_downloaded =
                    ChapterInfo::get_is_downloaded(app, &comic_title, &chapter_title);
                let chapter_info = ChapterInfo {
                    comic_id: comic.id,
                    comic_title: comic_title.clone(),
                    chapter_id,
                    chapter_title,
                    author: comic.author.clone(),
                    is_downloaded: Some(is_downloaded),
                    order,
                };
                Some(chapter_info)
            })
            .collect();
        // 如果没有章节信息，就添加一个默认的章节信息
        if chapter_infos.is_empty() {
            chapter_infos.push(ChapterInfo {
                comic_id: comic.id,
                comic_title: comic_title.clone(),
                chapter_id: comic.id,
                chapter_title: "第1话".to_owned(),
                author: comic.author.clone(),
                is_downloaded: Some(false),
                order: 1,
            });
        }

        let is_downloaded = Self::get_is_downloaded(app, &comic.name);

        Self {
            id: comic.id,
            name: comic.name,
            addtime: comic.addtime,
            description: comic.description,
            total_views: comic.total_views,
            likes: comic.likes,
            chapter_infos,
            series_id: comic.series_id,
            comment_total: comic.comment_total,
            author: comic.author,
            tags: comic.tags,
            works: comic.works,
            actors: comic.actors,
            related_list: comic.related_list,
            liked: comic.liked,
            is_favorite: comic.is_favorite,
            is_aids: comic.is_aids,
            is_downloaded: Some(is_downloaded),
        }
    }

    fn get_is_downloaded(app: &AppHandle, comic_title: &str) -> bool {
        Self::get_comic_download_dir(app, comic_title).exists()
    }

    // 这里脱裤子放屁，是为了后期方便扩展，例如给漫画目录加上作者名、id等
    pub fn get_comic_download_dir(app: &AppHandle, comic_title: &str) -> PathBuf {
        let comic_dir_name = Self::comic_dir_name(app, comic_title);
        app.state::<RwLock<Config>>()
            .read()
            .download_dir
            .join(comic_dir_name)
    }

    fn comic_dir_name(_app: &AppHandle, comic_title: &str) -> String {
        filename_filter(comic_title)
    }
}
