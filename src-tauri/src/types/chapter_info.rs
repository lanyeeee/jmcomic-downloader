use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChapterInfo {
    pub chapter_id: i64,
    pub chapter_title: String,
    pub order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_downloaded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_download_dir: Option<PathBuf>,
}

impl ChapterInfo {
    pub fn get_chapter_download_dir_name(&self) -> anyhow::Result<String> {
        let chapter_download_dir = self
            .chapter_download_dir
            .as_ref()
            .context("`chapter_download_dir`字段为`None`")?;

        let chapter_download_dir_name = chapter_download_dir
            .file_name()
            .context(format!(
                "获取`{}`的目录名失败",
                chapter_download_dir.display()
            ))?
            .to_string_lossy()
            .to_string();

        Ok(chapter_download_dir_name)
    }
}
