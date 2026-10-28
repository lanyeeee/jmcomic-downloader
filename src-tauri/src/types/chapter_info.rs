use std::{collections::HashMap, path::PathBuf};

use eyre::{eyre, OptionExt, WrapErr};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;
use tracing::instrument;

use crate::{extensions::AppHandleExt, utils};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ChapterInfo {
    pub chapter_id: i64,
    pub chapter_title: String,
    pub order: i64,
    /// 是否曾导出过 PDF
    pub is_pdf_exported: bool,
    /// 是否曾导出过 CBZ
    pub is_cbz_exported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_downloaded: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_download_dir: Option<PathBuf>,
}

impl ChapterInfo {
    #[instrument(
        level = "error",
        skip_all,
        fields(
            chapter_id = self.chapter_id,
            chapter_title = self.chapter_title,
            order = self.order
        )
    )]
    pub fn get_chapter_download_dir_name(&self) -> eyre::Result<String> {
        let chapter_download_dir = self
            .chapter_download_dir
            .as_ref()
            .ok_or_eyre("`chapter_download_dir`字段为`None`")?;

        let chapter_download_dir_name = chapter_download_dir
            .file_name()
            .ok_or_eyre(format!(
                "获取`{}`的目录名失败",
                chapter_download_dir.display()
            ))?
            .to_string_lossy()
            .to_string();

        Ok(chapter_download_dir_name)
    }

    #[instrument(
        level = "error",
        skip_all,
        fields(
            chapter_id = self.chapter_id,
            chapter_title = self.chapter_title,
            order = self.order
        )
    )]
    pub fn save_chapter_metadata(&self) -> eyre::Result<()> {
        let mut chapter_info = self.clone();
        // 将 is_downloaded 和 chapter_download_dir 字段设置为 None，
        // 这样能使这些字段在序列化时被忽略。
        chapter_info.is_downloaded = None;
        chapter_info.chapter_download_dir = None;

        let chapter_download_dir = self
            .chapter_download_dir
            .as_ref()
            .ok_or_eyre("`chapter_download_dir`字段为`None`")?;
        let metadata_path = chapter_download_dir.join("章节元数据.json");

        std::fs::create_dir_all(chapter_download_dir)
            .wrap_err(format!("创建目录`{}`失败", chapter_download_dir.display()))?;

        let chapter_json = serde_json::to_string_pretty(&chapter_info)
            .wrap_err("将ChapterInfo序列化为json失败")?;

        std::fs::write(&metadata_path, chapter_json)
            .wrap_err(format!("写入文件`{}`失败", metadata_path.display()))?;

        Ok(())
    }

    #[instrument(
        level = "error",
        skip_all,
        fields(
            comic_id = fmt_params.comic_id,
            comic_title = fmt_params.comic_title,
            author = fmt_params.author,
            chapter_id = fmt_params.chapter_id,
            chapter_title = fmt_params.chapter_title,
            order = fmt_params.order
        )
    )]
    pub fn get_chapter_download_dir_by_fmt(
        app: &AppHandle,
        fmt_params: &DirFmtParams,
    ) -> eyre::Result<PathBuf> {
        use strfmt::strfmt;

        let json_value =
            serde_json::to_value(fmt_params).wrap_err("将DirFmtParams转为serde_json::Value失败")?;

        let json_map = json_value
            .as_object()
            .ok_or_eyre("DirFmtParams不是JSON对象")?;

        let vars: HashMap<String, String> = json_map
            .iter()
            .map(|(k, v)| {
                let value = match v {
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                };
                (k.clone(), value)
            })
            .collect();

        let (download_dir, dir_fmt) = {
            let config = app.get_config();
            let config = config.read();
            (config.download_dir.clone(), config.dir_fmt.clone())
        };

        let dir_fmt_parts: Vec<&str> = dir_fmt.split('/').collect();

        let mut dir_names = Vec::new();
        for fmt in dir_fmt_parts {
            let dir_name = strfmt(fmt, &vars).wrap_err("格式化目录名失败")?;
            let dir_name = utils::filename_filter(&dir_name);
            if !dir_name.is_empty() {
                dir_names.push(dir_name);
            }
        }

        if dir_names.len() < 2 {
            let err_msg =
                "配置中的下载目录格式至少要有两个层级，例如：{comic_title}/{chapter_title}";
            return Err(eyre!(err_msg));
        }

        let mut chapter_download_dir = download_dir;
        for dir_name in dir_names {
            chapter_download_dir = chapter_download_dir.join(dir_name);
        }

        Ok(chapter_download_dir)
    }

    #[instrument(
        level = "error",
        skip_all,
        fields(
            chapter_id = self.chapter_id,
            chapter_title = self.chapter_title,
            order = self.order
        )
    )]
    pub fn get_temp_download_dir(&self) -> eyre::Result<PathBuf> {
        let chapter_download_dir = self
            .chapter_download_dir
            .as_ref()
            .ok_or_eyre("`chapter_download_dir`字段为`None`")?;

        let chapter_download_dir_name = self
            .get_chapter_download_dir_name()
            .wrap_err("获取章节下载目录名失败")?;

        let parent = chapter_download_dir.parent().ok_or_eyre(format!(
            "`{}`的父目录不存在",
            chapter_download_dir.display()
        ))?;

        let temp_download_dir = parent.join(format!(".下载中-{chapter_download_dir_name}"));
        Ok(temp_download_dir)
    }
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize, Type)]
pub struct DirFmtParams {
    pub comic_id: i64,
    pub comic_title: String,
    pub author: String,
    pub chapter_id: i64,
    pub chapter_title: String,
    pub order: i64,
}
