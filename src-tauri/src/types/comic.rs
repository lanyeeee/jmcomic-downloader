use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

use crate::{
    config::Config,
    extensions::WalkDirEntryExt,
    responses::{GetComicRespData, RelatedListRespData},
    utils,
};

use super::ChapterInfo;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_field_names)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comic_download_dir: Option<PathBuf>,
}

impl Comic {
    pub fn from_comic_resp_data(app: &AppHandle, comic: GetComicRespData) -> anyhow::Result<Comic> {
        let mut chapter_infos: Vec<ChapterInfo> = comic
            .series
            .into_iter()
            .enumerate()
            .filter_map(|(index, s)| {
                let chapter_id = s.id.parse().ok()?;
                #[allow(clippy::cast_possible_wrap)]
                let order = (index + 1) as i64;
                let mut chapter_title = format!("第{order}话");
                if !s.name.is_empty() {
                    chapter_title.push_str(&format!(" {}", &s.name));
                }
                let chapter_info = ChapterInfo {
                    chapter_id,
                    chapter_title,
                    order,
                    is_downloaded: None,
                    chapter_download_dir: None,
                };
                Some(chapter_info)
            })
            .collect();
        // 如果没有章节信息，就添加一个默认的章节信息
        if chapter_infos.is_empty() {
            chapter_infos.push(ChapterInfo {
                chapter_id: comic.id,
                chapter_title: "第1话".to_owned(),
                order: 1,
                is_downloaded: None,
                chapter_download_dir: None,
            });
        }

        let mut comic = Comic {
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
            is_downloaded: None,
            comic_download_dir: None,
        };

        let id_to_dir_map =
            utils::create_id_to_dir_map(app).context("创建漫画ID到下载目录映射失败")?;

        comic
            .update_fields(&id_to_dir_map)
            .context(format!("`{}`更新Comic的字段失败", comic.name))?;

        Ok(comic)
    }

    pub fn update_fields(&mut self, id_to_dir_map: &HashMap<i64, PathBuf>) -> anyhow::Result<()> {
        if let Some(download_dir) = id_to_dir_map.get(&self.id) {
            self.comic_download_dir = Some(download_dir.clone());
            self.is_downloaded = Some(true);

            self.update_chapter_infos_fields()
                .context("更新章节信息字段失败")?;
        }

        Ok(())
    }

    pub fn from_metadata(metadata_path: &Path) -> anyhow::Result<Comic> {
        let comic_json = std::fs::read_to_string(metadata_path)
            .context(format!("读取`{}`失败", metadata_path.display()))?;
        let mut comic = serde_json::from_str::<Comic>(&comic_json).context(format!(
            "将`{}`反序列化为Comic失败",
            metadata_path.display()
        ))?;
        // 来自元数据的章节信息没有`download_dir`和`is_downloaded`字段，需要更新
        let parent = metadata_path
            .parent()
            .context(format!("`{}`没有父目录", metadata_path.display()))?;
        comic.comic_download_dir = Some(parent.to_path_buf());
        comic.is_downloaded = Some(true);

        comic
            .update_chapter_infos_fields()
            .context("更新章节信息字段失败")?;

        Ok(comic)
    }

    pub fn get_comic_download_dir_name(&self) -> anyhow::Result<String> {
        let comic_download_dir = self
            .comic_download_dir
            .as_ref()
            .context("`comic_download_dir`字段为`None`")?;

        let comic_download_dir_name = comic_download_dir
            .file_name()
            .context(format!(
                "获取`{}`的目录名失败",
                comic_download_dir.display()
            ))?
            .to_string_lossy()
            .to_string();

        Ok(comic_download_dir_name)
    }

    pub fn get_comic_export_dir(&self, app: &AppHandle) -> anyhow::Result<PathBuf> {
        let (download_dir, export_dir) = {
            let config = app.state::<RwLock<Config>>();
            let config = config.read();
            (config.download_dir.clone(), config.export_dir.clone())
        };

        let Some(comic_download_dir) = self.comic_download_dir.clone() else {
            return Err(anyhow!("`comic_download_dir`字段为`None`"));
        };

        let relative_dir = comic_download_dir
            .strip_prefix(&download_dir)
            .context(format!(
                "无法从路径`{}`中移除前缀`{}`",
                comic_download_dir.display(),
                download_dir.display()
            ))?;

        let comic_export_dir = export_dir.join(relative_dir);
        Ok(comic_export_dir)
    }

    fn update_chapter_infos_fields(&mut self) -> anyhow::Result<()> {
        let Some(comic_download_dir) = &self.comic_download_dir else {
            return Err(anyhow!("`comic_download_dir`字段为`None`"));
        };

        if !comic_download_dir.exists() {
            return Ok(());
        }

        for entry in WalkDir::new(comic_download_dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.is_chapter_metadata() {
                continue;
            }

            let metadata_path = entry.path();

            let metadata_str = std::fs::read_to_string(metadata_path)
                .context(format!("读取`{}`失败", metadata_path.display()))?;

            let chapter_json: serde_json::Value =
                serde_json::from_str(&metadata_str).context(format!(
                    "将`{}`反序列化为serde_json::Value失败",
                    metadata_path.display()
                ))?;

            let chapter_id = chapter_json
                .get("chapterId")
                .and_then(serde_json::Value::as_i64)
                .context(format!("`{}`没有`chapterId`字段", metadata_path.display()))?;

            if let Some(chapter_info) = self
                .chapter_infos
                .iter_mut()
                .find(|chapter| chapter.chapter_id == chapter_id)
            {
                let parent = metadata_path
                    .parent()
                    .context(format!("`{}`没有父目录", metadata_path.display()))?;
                chapter_info.chapter_download_dir = Some(parent.to_path_buf());
                chapter_info.is_downloaded = Some(true);
            }
        }
        Ok(())
    }
}
