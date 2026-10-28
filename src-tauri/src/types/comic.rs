use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use eyre::{eyre, OptionExt, WrapErr};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tracing::instrument;
use walkdir::WalkDir;

use crate::{
    extensions::WalkDirEntryExt,
    responses::{GetComicRespData, RelatedListRespData},
    utils,
};

use super::{ChapterInfo, DirFmtParams};

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
    #[instrument(
        level = "error",
        skip_all,
        fields(comic_id = comic.id, comic_title = comic.name)
    )]
    pub fn from_comic_resp_data(app: &AppHandle, comic: GetComicRespData) -> eyre::Result<Comic> {
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
                    #[allow(clippy::format_push_string)]
                    chapter_title.push_str(&format!(" {}", &s.name));
                }
                let chapter_info = ChapterInfo {
                    chapter_id,
                    chapter_title,
                    order,
                    is_pdf_exported: false,
                    is_cbz_exported: false,
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
                is_pdf_exported: false,
                is_cbz_exported: false,
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

        let id_to_dir_map = utils::create_id_to_dir_map(app)?;

        // TODO: 这是为了兼容v0.15.4及之前的版本，后续需要移除，计划在v0.17.0之后移除
        if let Some(comic_download_dir) = id_to_dir_map.get(&comic.id) {
            comic
                .create_chapter_metadata_for_old_version(comic_download_dir)
                .wrap_err("为旧版本创建章节元数据失败")?;
        }

        comic.update_fields(&id_to_dir_map)?;

        Ok(comic)
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    pub fn update_fields(&mut self, id_to_dir_map: &HashMap<i64, PathBuf>) -> eyre::Result<()> {
        if let Some(comic_download_dir) = id_to_dir_map.get(&self.id) {
            self.comic_download_dir = Some(comic_download_dir.clone());
            self.is_downloaded = Some(true);

            self.update_chapter_infos_fields()
                .wrap_err("更新章节信息字段失败")?;
        }

        Ok(())
    }

    #[instrument(level = "error", skip_all, fields(metadata_path = %metadata_path.display()))]
    pub fn from_metadata(metadata_path: &Path) -> eyre::Result<Comic> {
        let comic_json = std::fs::read_to_string(metadata_path)?;
        let mut comic = serde_json::from_str::<Comic>(&comic_json)
            .wrap_err("将元数据文件反序列化为Comic失败")?;
        // 来自元数据的章节信息没有`download_dir`和`is_downloaded`字段，需要更新
        let parent = metadata_path
            .parent()
            .ok_or_eyre(format!("`{}`没有父目录", metadata_path.display()))?;
        let comic_download_dir = parent.to_path_buf();

        // TODO: 这是为了兼容v0.15.4及之前的版本，后续需要移除，计划在v0.17.0之后移除
        comic
            .create_chapter_metadata_for_old_version(&comic_download_dir)
            .wrap_err("为旧版本创建章节元数据失败")?;

        comic.comic_download_dir = Some(comic_download_dir);
        comic.is_downloaded = Some(true);

        comic.update_chapter_infos_fields()?;

        Ok(comic)
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    pub fn ensure_download_dir_fields(&mut self, app: &AppHandle) -> eyre::Result<()> {
        if self.has_download_dir_fields() {
            return Ok(());
        }

        self.update_download_dir_fields_by_fmt(app)
    }

    pub fn has_download_dir_fields(&self) -> bool {
        let comic_download_dir_ready = self.comic_download_dir.is_some();
        let chapter_download_dir_ready = self
            .chapter_infos
            .iter()
            .all(|chapter| chapter.chapter_download_dir.is_some());

        comic_download_dir_ready && chapter_download_dir_ready
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    pub fn save_comic_metadata(&self) -> eyre::Result<()> {
        let mut comic = self.clone();
        // 将漫画的is_downloaded和comic_download_dir字段设置为None
        // 这样能使这些字段在序列化时被忽略
        comic.is_downloaded = None;
        comic.comic_download_dir = None;
        for chapter in &mut comic.chapter_infos {
            // 将章节的is_downloaded和chapter_download_dir字段设置为None
            // 这样能使这些字段在序列化时被忽略
            chapter.is_downloaded = None;
            chapter.chapter_download_dir = None;
        }

        let comic_download_dir = self
            .comic_download_dir
            .as_ref()
            .ok_or_eyre("`comic_download_dir`字段为`None`")?;
        let metadata_path = comic_download_dir.join("元数据.json");

        std::fs::create_dir_all(comic_download_dir)
            .wrap_err(format!("创建目录`{}`失败", comic_download_dir.display()))?;

        let comic_json =
            serde_json::to_string_pretty(&comic).wrap_err("将Comic序列化为json失败")?;

        std::fs::write(&metadata_path, comic_json)
            .wrap_err(format!("写入文件`{}`失败", metadata_path.display()))?;

        Ok(())
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    pub fn get_cover_path(&self) -> eyre::Result<PathBuf> {
        let comic_download_dir = self
            .comic_download_dir
            .as_ref()
            .ok_or_eyre("`comic_download_dir`字段为`None`")?;

        let cover_path = comic_download_dir.join("cover.jpg");

        Ok(cover_path)
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    pub fn update_download_dir_fields_by_fmt(&mut self, app: &AppHandle) -> eyre::Result<()> {
        if self.chapter_infos.is_empty() {
            return Err(eyre!("没有章节信息，无法更新下载目录字段"));
        }

        let author = self.author.join(", ");
        let mut first_chapter_download_dir = None;

        for chapter_info in &mut self.chapter_infos {
            let chapter_title = &chapter_info.chapter_title;

            let dir_fmt_params = DirFmtParams {
                comic_id: self.id,
                comic_title: self.name.clone(),
                author: author.clone(),
                chapter_id: chapter_info.chapter_id,
                chapter_title: chapter_info.chapter_title.clone(),
                order: chapter_info.order,
            };

            let chapter_download_dir =
                ChapterInfo::get_chapter_download_dir_by_fmt(app, &dir_fmt_params)
                    .wrap_err(format!("章节`{chapter_title}`根据fmt获取章节下载目录失败"))?;

            if first_chapter_download_dir.is_none() {
                first_chapter_download_dir = Some(chapter_download_dir.clone());
            }

            chapter_info.chapter_download_dir = Some(chapter_download_dir);
        }

        let Some(first_chapter_download_dir) = first_chapter_download_dir else {
            return Err(eyre!(
                "处理完所有章节后first_chapter_download_dir仍然为None"
            ));
        };

        let comic_download_dir = first_chapter_download_dir.parent().ok_or_eyre(format!(
            "第一个章节下载目录`{}`没有父目录",
            first_chapter_download_dir.display()
        ))?;

        self.comic_download_dir = Some(comic_download_dir.to_path_buf());

        Ok(())
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    fn update_chapter_infos_fields(&mut self) -> eyre::Result<()> {
        let Some(comic_download_dir) = &self.comic_download_dir else {
            return Err(eyre!("`comic_download_dir`字段为`None`"));
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
                .wrap_err(format!("读取`{}`失败", metadata_path.display()))?;

            let chapter_json: serde_json::Value =
                serde_json::from_str(&metadata_str).wrap_err(format!(
                    "将`{}`反序列化为serde_json::Value失败",
                    metadata_path.display()
                ))?;

            let chapter_id = chapter_json
                .get("chapterId")
                .and_then(serde_json::Value::as_i64)
                .ok_or_eyre(format!("`{}`没有`chapterId`字段", metadata_path.display()))?;

            if let Some(chapter_info) = self
                .chapter_infos
                .iter_mut()
                .find(|chapter| chapter.chapter_id == chapter_id)
            {
                let parent = metadata_path
                    .parent()
                    .ok_or_eyre(format!("`{}`没有父目录", metadata_path.display()))?;
                chapter_info.chapter_download_dir = Some(parent.to_path_buf());
                chapter_info.is_downloaded = Some(true);
                chapter_info.is_pdf_exported = chapter_json
                    .get("isPdfExported")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                chapter_info.is_cbz_exported = chapter_json
                    .get("isCbzExported")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
            }
        }
        Ok(())
    }

    #[instrument(level = "error", skip_all, fields(comic_id = self.id, comic_title = self.name))]
    fn create_chapter_metadata_for_old_version(
        &self,
        comic_download_dir: &Path,
    ) -> eyre::Result<()> {
        let mut chapter_dirs = HashSet::new();
        for entry in std::fs::read_dir(comic_download_dir)?.filter_map(Result::ok) {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if !file_type.is_dir() {
                continue;
            }
            chapter_dirs.insert(entry.path());
        }

        for chapter_info in &self.chapter_infos {
            let old_chapter_dir = comic_download_dir.join(&chapter_info.chapter_title);
            let old_chapter_dir_exists = chapter_dirs.contains(&old_chapter_dir);
            let old_chapter_metadata_exists = old_chapter_dir.join("章节元数据.json").exists();
            if old_chapter_dir_exists && !old_chapter_metadata_exists {
                // 如果旧版本的章节目录存在，但没有元数据文件，就创建一个
                let mut info = chapter_info.clone();
                info.chapter_download_dir = Some(old_chapter_dir);
                info.is_downloaded = Some(true);
                info.save_chapter_metadata()?;
            }
        }

        Ok(())
    }
}
