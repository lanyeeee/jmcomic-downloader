use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context};
use bytes::Bytes;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{DynamicImage, GenericImage, GenericImageView};
use parking_lot::RwLock;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;

use crate::config::Config;
use crate::extensions::AnyhowErrorToStringChain;
use crate::jm_client::JmClient;
use crate::types::{AsyncRwLock, ChapterInfo, DownloadFormat, ProxyMode};
use crate::{utils, DownloadEvent};

pub const IMAGE_DOMAIN: &str = "cdn-msp2.jmapiproxy2.cc";

/// 用于管理下载任务
///
/// 克隆 `DownloadManager` 的开销极小，性能开销几乎可以忽略不计。
/// 可以放心地在多个线程中传递和使用它的克隆副本。
///
/// 具体来说：
/// - `client`和`app`的克隆开销很小。
/// - 其他字段都被 `Arc` 包裹，这些字段的克隆操作仅仅是增加引用计数。
#[derive(Clone)]
pub struct DownloadManager {
    http_client: Arc<AsyncRwLock<ClientWithMiddleware>>,
    app: AppHandle,
    rt: Arc<Runtime>,
    sender: Arc<mpsc::Sender<ChapterInfo>>,
    urls_with_block_num_sem: Arc<Semaphore>,
    chapter_sem: Arc<Semaphore>,
    img_sem: Arc<Semaphore>,
    byte_per_sec: Arc<AtomicU64>,
    downloaded_image_count: Arc<AtomicU32>,
    total_image_count: Arc<AtomicU32>,
}

impl DownloadManager {
    pub fn new(app: AppHandle) -> Self {
        let http_client = create_http_client(&app);
        let http_client = Arc::new(AsyncRwLock::new(http_client));
        // 创建异步运行时
        let core_count = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .expect("DownloadManager::new: 获取CPU核心数失败");
        let rt = tokio::runtime::Builder::new_multi_thread()
            .max_blocking_threads(core_count)
            .enable_all()
            .build()
            .expect("DownloadManager::new: 创建Runtime失败");
        let (sender, receiver) = mpsc::channel::<ChapterInfo>(32);
        let manager = DownloadManager {
            http_client,
            app,
            rt: Arc::new(rt),
            sender: Arc::new(sender),
            urls_with_block_num_sem: Arc::new(Semaphore::new(10)), // 最多同时获取10个urls_with_block_num
            chapter_sem: Arc::new(Semaphore::new(3)),              // 最多同时下载3个章节
            img_sem: Arc::new(Semaphore::new(40)),                 // 最多同时下载40张图片
            byte_per_sec: Arc::new(AtomicU64::new(0)),
            downloaded_image_count: Arc::new(AtomicU32::new(0)),
            total_image_count: Arc::new(AtomicU32::new(0)),
        };
        // TODO: 改用tauri::async_runtime::spawn
        manager.rt.spawn(manager.clone().log_download_speed());
        manager.rt.spawn(manager.clone().receiver_loop(receiver));

        manager
    }

    pub async fn recreate_http_client(&self) {
        let http_client = create_http_client(&self.app);
        *self.http_client.write().await = http_client;
    }

    pub async fn submit_chapter(&self, chapter_info: ChapterInfo) -> anyhow::Result<()> {
        Ok(self.sender.send(chapter_info).await?)
    }

    // TODO: 换个函数名，如emit_download_speed_loop
    #[allow(clippy::cast_precision_loss)]
    async fn log_download_speed(self) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let byte_per_sec = self.byte_per_sec.swap(0, Ordering::Relaxed);
            let mega_byte_per_sec = byte_per_sec as f64 / 1024.0 / 1024.0;
            let speed = format!("{mega_byte_per_sec:.2}MB/s");
            // 发送总进度条下载速度事件
            let _ = DownloadEvent::OverallSpeed { speed }.emit(&self.app);
        }
    }

    async fn receiver_loop(self, mut receiver: Receiver<ChapterInfo>) {
        while let Some(chapter_info) = receiver.recv().await {
            let manager = self.clone();
            tokio::spawn(manager.process_chapter(chapter_info));
        }
    }

    //TODO: 这里不能用anyhow::Result<()>和`?`，否则会导致错误信息被忽略
    async fn process_chapter(self, chapter_info: ChapterInfo) -> anyhow::Result<()> {
        // 发送章节排队事件
        let _ = DownloadEvent::ChapterPending {
            chapter_id: chapter_info.chapter_id,
            comic_title: chapter_info.comic_title.clone(),
            chapter_title: chapter_info.chapter_title.clone(),
        }
        .emit(&self.app);
        // 创建临时下载目录
        let temp_download_dir = chapter_info.get_temp_download_dir(&self.app);
        std::fs::create_dir_all(&temp_download_dir)
            .context(format!("创建目录`{temp_download_dir:?}`失败"))?;
        // 从配置文件获取图片格式
        let download_format = self.app.state::<RwLock<Config>>().read().download_format;
        // 获取此章节每张图片的下载链接以及对应的block_num
        let urls_with_block_num = self
            .get_urls_with_block_num(chapter_info.chapter_id)
            .await?;
        // 总共需要下载的图片数量
        #[allow(clippy::cast_possible_truncation)]
        let total = urls_with_block_num.len() as u32;
        // 记录总共需要下载的图片数量
        self.total_image_count.fetch_add(total, Ordering::Relaxed);
        let downloaded_count = Arc::new(AtomicU32::new(0));
        let mut join_set = JoinSet::new();
        // 限制同时下载的章节数量
        let permit = self.chapter_sem.acquire().await?;
        // 发送下载开始事件
        let _ = DownloadEvent::ChapterStart {
            chapter_id: chapter_info.chapter_id,
            total,
        }
        .emit(&self.app);
        for (i, (url, block_num)) in urls_with_block_num.into_iter().enumerate() {
            let manager = self.clone();
            let chapter_id = chapter_info.chapter_id;
            let ext = download_format.as_str();
            let save_path = temp_download_dir.join(format!("{:03}.{ext}", i + 1));
            let url = url.clone();
            let downloaded_count = downloaded_count.clone();
            // 创建下载任务
            join_set.spawn(manager.download_image(
                url,
                save_path,
                download_format,
                chapter_id,
                block_num,
                downloaded_count,
            ));
        }
        // 逐一处理完成的下载任务
        while let Some(completed_task) = join_set.join_next().await {
            completed_task?;
            self.downloaded_image_count.fetch_add(1, Ordering::Relaxed);
            let downloaded_image_count = self.downloaded_image_count.load(Ordering::Relaxed);
            let total_image_count = self.total_image_count.load(Ordering::Relaxed);
            // 更新下载进度
            #[allow(clippy::cast_lossless)]
            let percentage = downloaded_image_count as f64 / total_image_count as f64 * 100.0;
            // 发送总进度条更新事件
            let _ = DownloadEvent::OverallUpdate {
                downloaded_image_count,
                total_image_count,
                percentage,
            }
            .emit(&self.app);
        }
        drop(permit);
        // 如果DownloadManager所有图片全部都已下载(无论成功或失败)，则清空下载进度
        let downloaded_image_count = self.downloaded_image_count.load(Ordering::Relaxed);
        let total_image_count = self.total_image_count.load(Ordering::Relaxed);
        if downloaded_image_count == total_image_count {
            self.downloaded_image_count.store(0, Ordering::Relaxed);
            self.total_image_count.store(0, Ordering::Relaxed);
        }
        // 检查此章节的图片是否全部下载成功
        let downloaded_count = downloaded_count.load(Ordering::Relaxed);
        // 此章节的图片未全部下载成功
        if downloaded_count != total {
            let err_msg = Some(format!(
                "总共有 {total} 张图片，但只下载了 {downloaded_count} 张"
            ));
            // 发送下载结束事件
            let _ = DownloadEvent::ChapterEnd {
                chapter_id: chapter_info.chapter_id,
                err_msg,
            }
            .emit(&self.app);
            return Ok(());
        }
        // 此章节的图片全部下载成功
        let err_msg = match self.rename_temp_download_dir(&chapter_info, &temp_download_dir) {
            Ok(()) => None,
            Err(err) => Some(err.to_string_chain()),
        };
        // 发送下载结束事件
        let _ = DownloadEvent::ChapterEnd {
            chapter_id: chapter_info.chapter_id,
            err_msg,
        }
        .emit(&self.app);
        Ok(())
    }

    fn rename_temp_download_dir(
        &self,
        chapter_info: &ChapterInfo,
        temp_download_dir: &PathBuf,
    ) -> anyhow::Result<()> {
        let chapter_download_dir = chapter_info.get_chapter_download_dir(&self.app);

        if chapter_download_dir.exists() {
            std::fs::remove_dir_all(&chapter_download_dir)
                .context(format!("删除 {chapter_download_dir:?} 失败"))?;
        }

        std::fs::rename(temp_download_dir, &chapter_download_dir).context(format!(
            "将 {temp_download_dir:?} 重命名为 {chapter_download_dir:?} 失败"
        ))?;

        Ok(())
    }

    async fn get_urls_with_block_num(&self, chapter_id: i64) -> anyhow::Result<Vec<(String, u32)>> {
        let jm_client = self.app.state::<JmClient>();
        // 限制同时获取urls_with_block_num的数量
        let _permit = self.urls_with_block_num_sem.acquire().await?;
        // TODO: 获取`scramble_id`与`chapter_resp_data`可以并发
        let scramble_id = jm_client.get_scramble_id(chapter_id).await?;
        let chapter_resp_data = jm_client.get_chapter(chapter_id).await?;
        // 构造图片下载链接
        let urls_with_block_num: Vec<(String, u32)> = chapter_resp_data
            .images
            .into_iter()
            .filter_map(|filename| {
                let file_path = Path::new(&filename);
                let ext = file_path.extension()?.to_str()?.to_lowercase();
                if ext != "webp" {
                    return None;
                }
                let filename_without_ext = file_path.file_stem()?.to_str()?;
                let block_num = calculate_block_num(scramble_id, chapter_id, filename_without_ext);
                let url = format!("https://{IMAGE_DOMAIN}/media/photos/{chapter_id}/{filename}");
                Some((url, block_num))
            })
            .collect();
        Ok(urls_with_block_num)
    }

    async fn download_image(
        self,
        url: String,
        save_path: PathBuf,
        download_format: DownloadFormat,
        chapter_id: i64,
        block_num: u32,
        downloaded_count: Arc<AtomicU32>,
    ) {
        // 获取下载图片的semaphore
        let permit = match self.img_sem.acquire().await.map_err(anyhow::Error::from) {
            Ok(permit) => permit,
            Err(err) => {
                let err = err.context("获取下载图片的semaphore失败");
                // 发送下载图片失败事件
                let _ = DownloadEvent::ImageError {
                    chapter_id,
                    url: url.clone(),
                    err_msg: err.to_string_chain(),
                }
                .emit(&self.app);
                return;
            }
        };
        // 成功获取semaphore后，开始下载图片
        let image_data = match self.get_image_bytes(&url).await {
            Ok(data) => data,
            Err(err) => {
                let err = err.context(format!("下载图片`{url}`失败"));
                // 发送下载图片失败事件
                let _ = DownloadEvent::ImageError {
                    chapter_id,
                    url: url.clone(),
                    err_msg: err.to_string_chain(),
                }
                .emit(&self.app);
                return;
            }
        };
        // 下载完成后释放semaphore
        drop(permit);
        // 保存图片，因为保存图片可能要进行图片拼接
        // 而图片拼接是CPU密集型操作，所以使用spawn_blocking，避免阻塞worker threads
        // TODO: 改用rayon + tokio::sync::oneshot
        let _ = tokio::task::spawn_blocking(move || {
            if let Err(err) = save_image(&save_path, download_format, block_num, &image_data) {
                let err = err.context(format!("保存图片`{url}`到`{save_path:?}`失败"));
                // 发送下载图片失败事件
                let _ = DownloadEvent::ImageError {
                    chapter_id,
                    url,
                    err_msg: err.to_string_chain(),
                }
                .emit(&self.app);
                return;
            }
            // 记录下载字节数
            self.byte_per_sec
                .fetch_add(image_data.len() as u64, Ordering::Relaxed);
            // 更新章节下载进度
            let downloaded_count = downloaded_count.fetch_add(1, Ordering::Relaxed) + 1;
            let save_path = save_path.to_string_lossy().to_string();
            // 发送下载图片成功事件
            let _ = DownloadEvent::ImageSuccess {
                chapter_id,
                url: save_path,
                current: downloaded_count,
            }
            .emit(&self.app);
        })
        .await;
    }

    // TODO: 把下载图片的逻辑移到JmClient中
    async fn get_image_bytes(&self, url: &str) -> anyhow::Result<Bytes> {
        let http_res = self.http_client.read().await.get(url).send().await?;

        let status = http_res.status();
        if status != StatusCode::OK {
            let text = http_res.text().await?;
            let err = anyhow!("下载图片`{url}`失败，预料之外的状态码: {text}");
            return Err(err);
        }

        let mut image_data = http_res.bytes().await?;

        if image_data.is_empty() {
            // 如果图片为空，说明jm那边缓存失效了，带上时间戳再次请求，以避免缓存
            let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let query = json!({"ts": ts});
            let http_res = self
                .http_client
                .read()
                .await
                .get(url)
                .query(&query)
                .send()
                .await?;

            let status = http_res.status();
            if status != StatusCode::OK {
                let text = http_res.text().await?;
                let err = anyhow!("下载图片`{url}`失败，预料之外的状态码: {text}");
                return Err(err);
            }

            image_data = http_res.bytes().await?;
        }

        Ok(image_data)
    }
}

fn calculate_block_num(scramble_id: i64, id: i64, filename: &str) -> u32 {
    if id < scramble_id {
        0
    } else if id < 268_850 {
        10
    } else {
        let x = if id < 421_926 { 10 } else { 8 };
        let s = format!("{id}{filename}");
        let s = utils::md5_hex(&s);
        let mut block_num = s.chars().last().unwrap() as u32;
        block_num %= x;
        block_num = block_num * 2 + 2;
        block_num
    }
}

pub fn create_http_client(app: &AppHandle) -> ClientWithMiddleware {
    let builder = reqwest::ClientBuilder::new();

    let proxy_mode = app.state::<RwLock<Config>>().read().proxy_mode.clone();
    let builder = match proxy_mode {
        ProxyMode::System => builder,
        ProxyMode::NoProxy => builder.no_proxy(),
        ProxyMode::Custom => {
            let config = app.state::<RwLock<Config>>();
            let config = config.read();
            let proxy_host = &config.proxy_host;
            let proxy_port = &config.proxy_port;
            let proxy_url = format!("http://{proxy_host}:{proxy_port}");

            match reqwest::Proxy::all(&proxy_url).map_err(anyhow::Error::from) {
                Ok(proxy) => builder.proxy(proxy),
                Err(err) => {
                    let err_title = format!("`DownloadManager`设置代理`{proxy_url}`失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);
                    builder
                }
            }
        }
    };

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(2);

    reqwest_middleware::ClientBuilder::new(builder.build().unwrap())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

fn save_image(
    save_path: &PathBuf,
    download_format: DownloadFormat,
    block_num: u32,
    image_data: &Bytes,
) -> anyhow::Result<()> {
    let mut src_img = image::load_from_memory(image_data).context("解码图片失败")?;
    // 如果无需拼接，直接根据格式保存图片
    if block_num == 0 {
        return save_image_by_format(&src_img, save_path, download_format);
    }
    // 否则拼接图片
    let stitched_image = stitch_image(&mut src_img, block_num)?;
    save_image_by_format(&stitched_image, save_path, download_format)
}

/// 根据格式保存图片
fn save_image_by_format(
    img: &DynamicImage,
    save_path: &PathBuf,
    format: DownloadFormat,
) -> anyhow::Result<()> {
    match format {
        DownloadFormat::Jpeg => {
            img.to_rgb8().save(save_path)?;
        }
        DownloadFormat::Png => {
            let png_file = std::fs::File::create(save_path)?;
            let buffered_file_writer = BufWriter::new(png_file);
            let encoder = PngEncoder::new_with_quality(
                buffered_file_writer,
                CompressionType::Best,
                FilterType::default(),
            );
            img.write_with_encoder(encoder)?;
        }
        DownloadFormat::Webp => {
            img.to_rgba8().save(save_path)?;
        }
    };
    Ok(())
}

/// 拼接图片
fn stitch_image(src_img: &mut DynamicImage, block_num: u32) -> anyhow::Result<DynamicImage> {
    // 如果block_num不为0，需要将图片拼接后再保存
    let (width, height) = src_img.dimensions();
    // 创建一张空的图片，尺寸与原图相同，用于拼接分块
    let mut stitched_image = image::ImageBuffer::new(width, height);
    // 计算原图像的高度除以num的余数
    let remainder_height = height % block_num;
    // 将图片切分为block_num块并拼接
    for i in 0..block_num {
        // 计算当前块的标准高度
        let mut block_height = height / block_num;
        // 计算源图像中当前块的Y轴起点位置
        let src_img_y_start = height - (block_height * (i + 1)) - remainder_height;
        // 计算目标图像中当前块的Y轴起点位置
        let mut dst_img_y_start = block_height * i;
        // 第一块需要加上余数高度，以确保拼接完整
        if i == 0 {
            block_height += remainder_height;
        } else {
            dst_img_y_start += remainder_height;
        }
        // 从原图裁剪出当前块
        let cropped_block = src_img.crop(0, src_img_y_start, width, block_height);
        // 将裁剪出的当前块复制到新图的对应位置
        stitched_image
            .copy_from(&cropped_block, 0, dst_img_y_start)
            .context("拼接图片失败")?;
    }

    Ok(DynamicImage::ImageRgba8(stitched_image))
}
