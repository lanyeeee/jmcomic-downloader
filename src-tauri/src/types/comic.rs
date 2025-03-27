use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::{
    config::Config,
    responses::{GetComicRespData, RelatedListRespData},
    utils,
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
}

impl Comic {
    pub fn from_comic_resp_data(app: &AppHandle, comic: GetComicRespData) -> Self {
        let comic_title = utils::filename_filter(&comic.name);
        let mut chapter_infos: Vec<ChapterInfo> = comic
            .series
            .into_iter()
            .filter_map(|s| {
                let chapter_id = s.id.parse().ok()?;
                let mut chapter_title = format!("第{}话", s.sort);
                if !s.name.is_empty() {
                    chapter_title.push_str(&format!(" {}", utils::filename_filter(&s.name)));
                }
                let is_downloaded = Self::get_is_downloaded(app, &comic_title, &chapter_title);
                let chapter_info = ChapterInfo {
                    comic_id: comic.id,
                    comic_title: comic_title.clone(),
                    chapter_id,
                    chapter_title,
                    is_downloaded,
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
                is_downloaded: false,
            });
        }

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
        }
    }

    fn get_is_downloaded(app: &AppHandle, comic_title: &str, chapter_title: &str) -> bool {
        let config = app.state::<RwLock<Config>>();
        let config = config.read();
        config
            .download_dir
            .join(comic_title)
            .join(chapter_title)
            .with_extension(config.archive_format.extension())
            .exists()
    }
}
