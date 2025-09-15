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

    pub fn save_chapter_metadata(&self) -> anyhow::Result<()> {
        let mut chapter_info = self.clone();
        // 将is_downloaded和chapter_download_dir字段设置为None
        // 这样能使这些字段在序列化时被忽略
        chapter_info.is_downloaded = None;
        chapter_info.chapter_download_dir = None;

        let chapter_download_dir = self
            .chapter_download_dir
            .as_ref()
            .context("`chapter_download_dir`字段为`None`")?;
        let metadata_path = chapter_download_dir.join("章节元数据.json");

        std::fs::create_dir_all(chapter_download_dir)
            .context(format!("创建目录`{}`失败", chapter_download_dir.display()))?;

        let chapter_json =
            serde_json::to_string_pretty(&chapter_info).context("将ChapterInfo序列化为json失败")?;

        std::fs::write(&metadata_path, chapter_json)
            .context(format!("写入文件`{}`失败", metadata_path.display()))?;

        Ok(())
    }
}
