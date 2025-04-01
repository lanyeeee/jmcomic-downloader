use std::{
    io::Write,
    sync::{atomic::AtomicU32, Arc},
};

use anyhow::{anyhow, Context};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tauri::AppHandle;
use tauri_specta::Event;
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    events::ExportCbzEvent,
    types::{Comic, ComicInfo},
    utils::filename_filter,
};

pub enum ExportArchive {
    Cbz,
}

impl ExportArchive {
    pub fn extension(&self) -> &str {
        match self {
            ExportArchive::Cbz => "cbz",
        }
    }
}

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
pub fn cbz(app: &AppHandle, comic: &Comic) -> anyhow::Result<()> {
    let comic_title = &comic.name.clone();
    let downloaded_chapter_infos = comic
        .chapter_infos
        .iter()
        .filter(|chapter_info| chapter_info.is_downloaded.unwrap_or(false))
        .collect::<Vec<_>>();
    // 生成格式化的xml
    let cfg = yaserde::ser::Config {
        perform_indent: true,
        ..Default::default()
    };
    let event_uuid = uuid::Uuid::new_v4().to_string();
    // 发送开始导出cbz事件
    let _ = ExportCbzEvent::Start {
        uuid: event_uuid.clone(),
        comic_title: comic_title.clone(),
        total: downloaded_chapter_infos.len() as u32,
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

    let extension = ExportArchive::Cbz.extension();
    let comic_export_dir = Comic::get_comic_export_dir(app, comic_title);
    let chapter_export_dir = comic_export_dir.join(ExportArchive::Cbz.extension());
    // 保证导出目录存在
    std::fs::create_dir_all(&chapter_export_dir).context(format!(
        "`{comic_title}`创建目录`{chapter_export_dir:?}`失败"
    ))?;
    // 并发处理
    let downloaded_chapter_infos = downloaded_chapter_infos.into_par_iter();
    downloaded_chapter_infos.try_for_each(|chapter_info| -> anyhow::Result<()> {
        let chapter_title = chapter_info.chapter_title.clone();
        let err_prefix = format!("`{comic_title} - {chapter_title}`");

        let chapter_download_dir = chapter_info.get_chapter_download_dir(app);
        // 生成ComicInfo
        let comic_info = ComicInfo::from(comic, chapter_info);
        // 序列化ComicInfo为xml
        let comic_info_xml = yaserde::ser::to_string_with_config(&comic_info, &cfg)
            .map_err(|err_msg| anyhow!("{err_prefix}序列化`ComicInfo.xml`失败: {err_msg}"))?;
        // 创建cbz文件
        let sanitized_chapter_title = filename_filter(&chapter_title);
        let zip_path = chapter_export_dir.join(format!("{sanitized_chapter_title}.{extension}"));
        let zip_file = std::fs::File::create(&zip_path)
            .context(format!("{err_prefix}创建文件`{zip_path:?}`失败"))?;
        let mut zip_writer = ZipWriter::new(zip_file);
        // 把ComicInfo.xml写入cbz
        zip_writer
            .start_file("ComicInfo.xml", SimpleFileOptions::default())
            .context(format!(
                "{err_prefix}在`{zip_path:?}`创建`ComicInfo.xml`失败"
            ))?;
        zip_writer
            .write_all(comic_info_xml.as_bytes())
            .context(format!("{err_prefix}写入`ComicInfo.xml`失败"))?;
        // 遍历下载目录，将文件写入cbz
        let image_entries = std::fs::read_dir(&chapter_download_dir)
            .context(format!(
                "{err_prefix}读取目录`{chapter_download_dir:?}`失败"
            ))?
            .filter_map(Result::ok);
        for image_entry in image_entries {
            let image_path = image_entry.path();
            if !image_path.is_file() {
                continue;
            }

            let filename = match image_path.file_name() {
                Some(name) => name.to_string_lossy(),
                None => continue,
            };
            // 将文件写入cbz
            zip_writer
                .start_file(&filename, SimpleFileOptions::default())
                .context(format!(
                    "{err_prefix}在`{zip_path:?}`创建`{filename:?}`失败"
                ))?;
            let mut file =
                std::fs::File::open(&image_path).context(format!("打开`{image_path:?}`失败"))?;
            std::io::copy(&mut file, &mut zip_writer).context(format!(
                "{err_prefix}将`{image_path:?}`写入`{zip_path:?}`失败"
            ))?;
        }

        zip_writer
            .finish()
            .context(format!("{err_prefix}关闭`{zip_path:?}`失败"))?;
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
    let _ = ExportCbzEvent::End { uuid: event_uuid }.emit(app);

    Ok(())
}
