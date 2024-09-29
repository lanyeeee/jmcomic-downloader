use std::fmt::Display;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::config::Config;
use crate::extensions::IgnoreRwLockPoison;
use crate::responses::{AlbumRespData, RelatedListRespData, SearchResp, SearchRespData};
use crate::utils;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchSort {
    Latest,
    View,
    Picture,
    Like,
}
impl SearchSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchSort::Latest => "mr",
            SearchSort::View => "mv",
            SearchSort::Picture => "mp",
            SearchSort::Like => "tf",
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Album {
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
impl Album {
    pub fn from_album_resp_data(app: &AppHandle, album: AlbumRespData) -> Self {
        let album_title = utils::filename_filter(&album.name);
        let mut chapter_infos: Vec<ChapterInfo> = album
            .series
            .into_iter()
            .filter_map(|s| {
                let chapter_id = s.id.parse().ok()?;
                let mut chapter_title = format!("第{}话", s.sort);
                if !s.name.is_empty() {
                    chapter_title.push_str(&format!(" {}", utils::filename_filter(&s.name)));
                }
                let download_dir = app
                    .state::<RwLock<Config>>()
                    .read_or_panic()
                    .download_dir
                    .join(&album_title)
                    .join(&chapter_title);
                let chapter_info = ChapterInfo {
                    album_id: album.id,
                    album_title: album_title.clone(),
                    chapter_id,
                    chapter_title,
                    is_downloaded: download_dir.exists(),
                };
                Some(chapter_info)
            })
            .collect();
        // 如果没有章节信息，就添加一个默认的章节信息
        if chapter_infos.is_empty() {
            chapter_infos.push(ChapterInfo {
                album_id: album.id,
                album_title: album_title.clone(),
                chapter_id: album.id,
                chapter_title: "第1话".to_owned(),
                is_downloaded: false,
            });
        }

        Self {
            id: album.id,
            name: album.name,
            addtime: album.addtime,
            description: album.description,
            total_views: album.total_views,
            likes: album.likes,
            chapter_infos,
            series_id: album.series_id,
            comment_total: album.comment_total,
            author: album.author,
            tags: album.tags,
            works: album.works,
            actors: album.actors,
            related_list: album.related_list,
            liked: album.liked,
            is_favorite: album.is_favorite,
            is_aids: album.is_aids,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChapterInfo {
    pub chapter_id: i64,
    pub chapter_title: String,
    pub album_id: i64,
    pub album_title: String,
    pub is_downloaded: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchResult {
    SearchRespData(SearchRespData),
    // 用Box包装AlbumRespData，因为Album比SearchRespData大得多
    // 如果不用Box包装，即使SearchResp的类型是SearchRespData，也会占用与AlbumRespData一样大的内存
    Album(Box<Album>),
}

impl SearchResult {
    pub fn from_search_resp(app: &AppHandle, search_resp: SearchResp) -> Self {
        match search_resp {
            SearchResp::SearchRespData(search_resp) => SearchResult::SearchRespData(search_resp),
            SearchResp::AlbumRespData(album_resp) => {
                let album = Album::from_album_resp_data(app, *album_resp);
                SearchResult::Album(Box::new(album))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum FavoriteSort {
    FavoriteTime,
    UpdateTime,
}

impl FavoriteSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            FavoriteSort::FavoriteTime => "mr",
            FavoriteSort::UpdateTime => "mp",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum DownloadFormat {
    Jpeg,
    Png,
    Webp,
}

impl DownloadFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            DownloadFormat::Jpeg => "jpg",
            DownloadFormat::Png => "png",
            DownloadFormat::Webp => "webp",
        }
    }
}
