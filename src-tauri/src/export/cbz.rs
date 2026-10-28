use std::{
    io::Write,
    path::Path,
    sync::{atomic::AtomicU32, Arc},
};

use eyre::{eyre, OptionExt, WrapErr};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tauri::AppHandle;
use tauri_specta::Event;
use tracing::instrument;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    config::ExportSkipMode,
    events::ExportCbzEvent,
    export::{
        build_export_targets, get_downloaded_chapters, get_downloaded_chapters_by_ids,
        get_image_paths, ComicExportLockGuard, ExportFormat, ExportTarget,
    },
    extensions::{AppHandleExt, EyreReportToMessage},
    types::{ChapterInfo, Comic, ComicInfo},
};

struct CbzErrorEventGuard {
    uuid: String,
    app: AppHandle,
    success: bool,
}

impl Drop for CbzErrorEventGuard {
    fn drop(&mut self) {
        if self.success {
            return;
        }

        let uuid = self.uuid.clone();
        let _ = ExportCbzEvent::Error { uuid }.emit(&self.app);
    }
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
#[instrument(level = "error", skip_all, fields(comic_id = comic.id, comic_title = comic.name))]
pub fn cbz(app: &AppHandle, comic: &Comic) -> eyre::Result<()> {
    let comic_id = comic.id;
    let comic_title = &comic.name;
    let export_lock = app.get_export_lock().inner().clone();

    if !export_lock.try_acquire(comic_id) {
        return Err(eyre!("漫画`{comic_title}`正在导出，请稍后再试"));
    }

    let _guard = ComicExportLockGuard {
        lock: export_lock.clone(),
        comic_id,
    };

    let skip_mode = app.get_config().read().export_skip_mode;
    let downloaded_chapters = get_downloaded_chapters(&comic.chapter_infos);

    cbz_internal(app, comic, downloaded_chapters, skip_mode)
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
#[instrument(level = "error", skip_all, fields(comic_id = comic.id, comic_title = comic.name))]
pub fn cbz_chapters(app: &AppHandle, comic: &Comic, chapter_ids: Vec<i64>) -> eyre::Result<()> {
    let comic_id = comic.id;
    let comic_title = &comic.name;
    let export_lock = app.get_export_lock().inner().clone();

    if !export_lock.try_acquire(comic_id) {
        return Err(eyre!("漫画`{comic_title}`正在导出，请稍后再试"));
    }

    let _guard = ComicExportLockGuard {
        lock: export_lock.clone(),
        comic_id,
    };

    let downloaded_chapters = get_downloaded_chapters_by_ids(&comic.chapter_infos, &chapter_ids);
    cbz_internal(app, comic, downloaded_chapters, ExportSkipMode::None)
}

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::too_many_lines)]
#[instrument(level = "error", skip_all, fields(skip_mode = ?skip_mode))]
fn cbz_internal(
    app: &AppHandle,
    comic: &Comic,
    downloaded_chapters: Vec<ChapterInfo>,
    skip_mode: ExportSkipMode,
) -> eyre::Result<()> {
    let export_targets = build_export_targets(app, comic, downloaded_chapters, ExportFormat::Cbz)?;

    // 章节字段只能出现在格式的最后一层，所以所有章节都导出到同一个目录
    let Some(first_target) = export_targets.first() else {
        return Ok(());
    };
    let cbz_export_dir = first_target
        .export_path
        .parent()
        .ok_or_eyre(format!(
            "`{}`没有父目录",
            first_target.export_path.display()
        ))?
        .to_path_buf();

    // 生成格式化的xml
    let cfg = yaserde::ser::Config {
        perform_indent: true,
        ..Default::default()
    };
    let event_uuid = uuid::Uuid::new_v4().to_string();
    // 发送开始导出cbz事件
    let _ = ExportCbzEvent::Start {
        uuid: event_uuid.clone(),
        comic_title: comic.name.clone(),
        total: export_targets.len() as u32,
    }
    .emit(app);
    // 如果success为false，drop时发送Error事件
    let mut error_event_guard = CbzErrorEventGuard {
        uuid: event_uuid.clone(),
        app: app.clone(),
        success: false,
    };
    // 用来记录导出进度
    let current = Arc::new(AtomicU32::new(0));

    // 保证导出目录存在
    std::fs::create_dir_all(&cbz_export_dir)
        .wrap_err(format!("创建目录`{}`失败", cbz_export_dir.display()))?;
    // 先把封面拷贝到导出目录(如果有)
    if let Err(err) = copy_cover(comic, &cbz_export_dir) {
        let err_title = "将封面拷贝到导出目录失败";
        let message = err.to_message();
        tracing::error!(err_title, message);
    }
    // 并发处理
    let current_span = tracing::Span::current();
    let export_targets = export_targets.into_par_iter();
    export_targets.try_for_each(|target| -> eyre::Result<()> {
        let ExportTarget {
            mut chapter_info,
            export_path: zip_path,
        } = target;

        let _enter = current_span.enter();
        let span = tracing::error_span!(
            "export_cbz_rayon",
            comic_id = comic.id,
            comic_title = comic.name,
            chapter_id = chapter_info.chapter_id,
            chapter_title = chapter_info.chapter_title
        );
        let _enter = span.enter();

        // 获取章节下载目录
        let chapter_download_dir = chapter_info
            .chapter_download_dir
            .as_ref()
            .ok_or_eyre("`chapter_download_dir`字段为`None`")?;

        // 跳过逻辑
        let should_skip = match skip_mode {
            ExportSkipMode::SkipExported if chapter_info.is_cbz_exported => true,
            ExportSkipMode::SkipExisting if zip_path.exists() => true,
            _ => false,
        };

        if should_skip {
            // 更新进度
            let current = current.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let _ = ExportCbzEvent::Progress {
                uuid: event_uuid.clone(),
                current,
            }
            .emit(app);
            return Ok(());
        }

        // 生成ComicInfo
        let comic_info = ComicInfo::from(comic, &chapter_info);
        // 序列化ComicInfo为xml
        let comic_info_xml = yaserde::ser::to_string_with_config(&comic_info, &cfg)
            .map_err(|err_msg| eyre!("序列化`ComicInfo.xml`失败: {err_msg}"))?;

        // 创建cbz文件
        let zip_file = std::fs::File::create(&zip_path)
            .wrap_err(format!("创建文件`{}`失败", zip_path.display()))?;
        let mut zip_writer = ZipWriter::new(zip_file);
        // 把ComicInfo.xml写入cbz
        zip_writer
            .start_file("ComicInfo.xml", SimpleFileOptions::default())
            .wrap_err(format!("在`{}`创建`ComicInfo.xml`失败", zip_path.display()))?;
        zip_writer
            .write_all(comic_info_xml.as_bytes())
            .wrap_err("写入`ComicInfo.xml`失败")?;

        let image_paths = get_image_paths(chapter_download_dir, false).wrap_err(format!(
            "获取`{}`中的图片失败",
            chapter_download_dir.display()
        ))?;

        for image_path in image_paths {
            let filename = image_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_eyre(format!("获取`{}`的文件名失败", image_path.display()))?;
            // 将文件写入cbz
            zip_writer
                .start_file(filename, SimpleFileOptions::default())
                .wrap_err(format!("在`{}`创建`{filename}`失败", zip_path.display()))?;
            let mut file = std::fs::File::open(&image_path)
                .wrap_err(format!("打开`{}`失败", image_path.display()))?;
            std::io::copy(&mut file, &mut zip_writer).wrap_err(format!(
                "将`{}`写入`{}`失败",
                image_path.display(),
                zip_path.display()
            ))?;
        }

        zip_writer
            .finish()
            .wrap_err(format!("关闭`{}`失败", zip_path.display()))?;

        // 更新章节导出状态
        chapter_info.is_cbz_exported = true;
        chapter_info.save_chapter_metadata()?;

        // 更新导出cbz的进度
        let current = current.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        // 发送导出cbz进度事件
        let _ = ExportCbzEvent::Progress {
            uuid: event_uuid.clone(),
            current,
        }
        .emit(app);

        Ok(())
    })?;
    // 标记为成功，后面drop时就不会发送Error事件
    error_event_guard.success = true;
    // 发送导出cbz完成事件
    let _ = ExportCbzEvent::End {
        uuid: event_uuid,
        comic_id: comic.id,
        chapter_export_dir: cbz_export_dir,
    }
    .emit(app);

    Ok(())
}

#[instrument(
    level = "error",
    skip_all,
    fields(
        comic_id = comic.id,
        comic_title = comic.name,
        chapter_export_dir = %chapter_export_dir.display()
    )
)]
fn copy_cover(comic: &Comic, chapter_export_dir: &Path) -> eyre::Result<()> {
    let src_cover_path = comic.get_cover_path().wrap_err("获取封面路径失败")?;
    let cover_filename = src_cover_path
        .file_name()
        .ok_or_eyre("获取封面的文件名失败")?;

    if src_cover_path.exists() {
        let dst_cover_path = chapter_export_dir.join(cover_filename);
        std::fs::copy(src_cover_path, dst_cover_path)?;
    }

    Ok(())
}
