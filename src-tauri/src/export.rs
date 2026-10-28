mod cbz;
mod pdf;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};

pub use cbz::{cbz, cbz_chapters};
use eyre::{eyre, OptionExt, WrapErr};
use parking_lot::Mutex;
pub use pdf::{pdf, pdf_chapters};
use serde::Serialize;
use tauri::AppHandle;
use tracing::instrument;

use crate::{
    extensions::{AppHandleExt, PathIsImg},
    types::{ChapterInfo, Comic},
    utils,
};

/// 导出互斥锁管理器，确保同一漫画的导出操作串行执行
#[derive(Debug, Clone, Default)]
pub struct ComicExportLock {
    /// 正在导出的漫画 ID 集合
    locked_comic_ids: Arc<Mutex<HashSet<i64>>>,
}

impl ComicExportLock {
    pub fn new() -> Self {
        Self {
            locked_comic_ids: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// 尝试获取漫画导出锁，返回是否成功
    pub fn try_acquire(&self, comic_id: i64) -> bool {
        let mut locked = self.locked_comic_ids.lock();
        if locked.contains(&comic_id) {
            return false;
        }
        locked.insert(comic_id);
        true
    }

    /// 释放漫画导出锁
    pub fn release(&self, comic_id: i64) {
        self.locked_comic_ids.lock().remove(&comic_id);
    }
}

pub struct ComicExportLockGuard {
    lock: ComicExportLock,
    comic_id: i64,
}

impl Drop for ComicExportLockGuard {
    fn drop(&mut self) {
        self.lock.release(self.comic_id);
    }
}

/// 导出格式
#[derive(Debug, Copy, Clone)]
pub enum ExportFormat {
    Pdf,
    Cbz,
}

impl ExportFormat {
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Cbz => "cbz",
        }
    }
}

/// 获取已下载的章节
fn get_downloaded_chapters(chapter_infos: &[ChapterInfo]) -> Vec<ChapterInfo> {
    chapter_infos
        .iter()
        .filter(|chapter_info| chapter_info.is_downloaded.unwrap_or(false))
        .cloned()
        .collect()
}

/// 根据章节 ID 列表获取已下载的章节
fn get_downloaded_chapters_by_ids(
    chapter_infos: &[ChapterInfo],
    chapter_ids: &[i64],
) -> Vec<ChapterInfo> {
    let chapter_id_set: HashSet<_> = chapter_ids.iter().copied().collect();
    chapter_infos
        .iter()
        .filter(|chapter_info| {
            chapter_info.is_downloaded.unwrap_or(false)
                && chapter_id_set.contains(&chapter_info.chapter_id)
        })
        .cloned()
        .collect()
}

#[instrument(
    level = "error",
    skip_all,
    fields(images_dir = %images_dir.display(), must_be_common_img = must_be_common_img)
)]
fn get_image_paths(images_dir: &Path, must_be_common_img: bool) -> eyre::Result<Vec<PathBuf>> {
    let mut image_paths: Vec<PathBuf> = std::fs::read_dir(images_dir)
        .wrap_err(format!("读取目录`{}`失败", images_dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            if must_be_common_img {
                path.is_common_img()
            } else {
                path.is_img()
            }
        })
        .collect();
    image_paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    Ok(image_paths)
}

/// 章节及其导出文件路径
struct ExportTarget {
    chapter_info: ChapterInfo,
    export_path: PathBuf,
}

/// `导出目录格式`中任意层级都能用的字段
const EXPORT_FMT_COMMON_FIELDS: [&str; 4] = ["comic_id", "comic_title", "author", "export_format"];
/// `导出目录格式`中只能用在最后一层的字段
const EXPORT_FMT_CHAPTER_FIELDS: [&str; 3] = ["chapter_id", "chapter_title", "order"];

#[derive(Debug, Serialize)]
struct ChapterExportFmtParams {
    comic_id: i64,
    comic_title: String,
    author: String,
    export_format: String,
    chapter_id: i64,
    chapter_title: String,
    order: i64,
}

/// 按`导出目录格式`计算每个章节的导出文件路径
///
/// 例如格式为`{export_format}/{comic_title}/{chapter_title}`，
/// 《蓦然回首》第1话导出为cbz的路径是`导出目录/cbz/蓦然回首/第1话.cbz`
#[instrument(
    level = "error",
    skip_all,
    fields(comic_id = comic.id, comic_title = comic.name, export_format = ?export_format)
)]
fn build_export_targets(
    app: &AppHandle,
    comic: &Comic,
    chapter_infos: Vec<ChapterInfo>,
    export_format: ExportFormat,
) -> eyre::Result<Vec<ExportTarget>> {
    let (export_dir, export_dir_fmt) = {
        let config = app.get_config();
        let config = config.read();
        (config.export_dir.clone(), config.export_dir_fmt.clone())
    };

    validate_export_dir_fmt(&export_dir_fmt)?;

    let author = comic.author.join(", ");
    let extension = export_format.extension();

    chapter_infos
        .into_iter()
        .map(|chapter_info| {
            let fmt_params = ChapterExportFmtParams {
                comic_id: comic.id,
                comic_title: comic.name.clone(),
                author: author.clone(),
                export_format: extension.to_string(),
                chapter_id: chapter_info.chapter_id,
                chapter_title: chapter_info.chapter_title.clone(),
                order: chapter_info.order,
            };
            let vars = to_fmt_vars(&fmt_params)?;
            let export_path = fmt_to_path(&export_dir, &export_dir_fmt, &vars, extension)
                .wrap_err(format!(
                    "按`导出目录格式`生成章节`{}`的导出路径失败",
                    chapter_info.chapter_title
                ))?;

            Ok(ExportTarget {
                chapter_info,
                export_path,
            })
        })
        .collect()
}

/// 章节字段只能出现在最后一层，这样同一部漫画的章节都导出到同一个目录
///
/// 最后一层必须包含章节字段，这样不同章节的导出文件不会重名
fn validate_export_dir_fmt(export_dir_fmt: &str) -> eyre::Result<()> {
    let mut fmt_parts: Vec<&str> = export_dir_fmt.split('/').collect();
    let last_fmt_part = fmt_parts.pop().unwrap_or_default();

    if find_used_field(last_fmt_part, &EXPORT_FMT_CHAPTER_FIELDS).is_none() {
        return Err(eyre!(
            "`导出目录格式`不合法，最后一层必须至少包含一个章节字段 {EXPORT_FMT_CHAPTER_FIELDS:?}"
        ));
    }

    for fmt_part in fmt_parts {
        if let Some(field) = find_used_field(fmt_part, &EXPORT_FMT_CHAPTER_FIELDS) {
            return Err(eyre!(
                "`导出目录格式`不合法，章节字段只能出现在最后一层，你当前在非最后一层使用了`{field}`。非最后一层只允许使用 {EXPORT_FMT_COMMON_FIELDS:?}"
            ));
        }
    }

    Ok(())
}

/// 返回`fmt_part`里用到的第一个字段，`{order}`和`{order:0>3}`都算用到了`order`
fn find_used_field<'a>(fmt_part: &str, fields: &[&'a str]) -> Option<&'a str> {
    fields.iter().copied().find(|field| {
        fmt_part.contains(&format!("{{{field}}}")) || fmt_part.contains(&format!("{{{field}:"))
    })
}

/// 把格式化参数转为`strfmt`用的变量表，字段名即变量名
fn to_fmt_vars(fmt_params: &impl Serialize) -> eyre::Result<HashMap<String, String>> {
    let json_value =
        serde_json::to_value(fmt_params).wrap_err("将格式化参数转为serde_json::Value失败")?;

    let json_map = json_value
        .as_object()
        .ok_or_eyre("格式化参数不是JSON对象")?;

    let vars = json_map
        .iter()
        .map(|(key, value)| {
            let value = match value {
                serde_json::Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            (key.clone(), value)
        })
        .collect();

    Ok(vars)
}

/// 用`vars`格式化`fmt`，在`export_dir`下生成文件路径
///
/// `/`分隔目录层级，最后一层是文件名(不含扩展名)
/// 例如`{comic_title}/{comic_title}`生成`export_dir/蓦然回首/蓦然回首.pdf`
fn fmt_to_path(
    export_dir: &Path,
    fmt: &str,
    vars: &HashMap<String, String>,
    extension: &str,
) -> eyre::Result<PathBuf> {
    use strfmt::strfmt;

    let mut fmt_parts: Vec<&str> = fmt.split('/').collect();
    let filename_fmt = fmt_parts.pop().unwrap_or_default();

    let mut path = export_dir.to_path_buf();
    for fmt_part in fmt_parts {
        let dir_name = strfmt(fmt_part, vars).wrap_err(format!("格式化`{fmt_part}`失败"))?;
        let dir_name = utils::filename_filter(&dir_name);
        if !dir_name.is_empty() {
            path.push(dir_name);
        }
    }

    let filename = strfmt(filename_fmt, vars).wrap_err(format!("格式化`{filename_fmt}`失败"))?;
    let filename = utils::filename_filter(&filename);
    if filename.is_empty() {
        return Err(eyre!("`{fmt}`的最后一层格式化后为空，无法作为文件名"));
    }
    path.push(format!("{filename}.{extension}"));

    Ok(path)
}
