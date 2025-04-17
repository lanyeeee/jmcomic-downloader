use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::Aes256;
use anyhow::{anyhow, Context};
use base64::engine::general_purpose;
use base64::Engine;
use bytes::Bytes;
use parking_lot::RwLock;
use reqwest::cookie::Jar;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::{Jitter, RetryTransientMiddleware};
use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::config::Config;
use crate::download_manager::IMAGE_DOMAIN;
use crate::extensions::AnyhowErrorToStringChain;
use crate::responses::{
    GetChapterRespData, GetComicRespData, GetFavoriteRespData, GetUserProfileRespData, JmResp,
    RedirectRespData, SearchResp, SearchRespData, ToggleFavoriteRespData,
};
use crate::types::{FavoriteSort, ProxyMode, SearchSort};
use crate::utils;

const APP_TOKEN_SECRET: &str = "18comicAPP";
const APP_TOKEN_SECRET_2: &str = "18comicAPPContent";
const APP_DATA_SECRET: &str = "185Hcomic3PAPP7R";
const APP_VERSION: &str = "1.7.5";

const API_DOMAIN: &str = "www.cdnblackmyth.club";

#[derive(Debug, Clone, PartialEq)]
enum ApiPath {
    Login,
    GetUserProfile,
    Search,
    GetComic,
    GetChapter,
    GetScrambleId,
    GetFavoriteFolder,
}
impl ApiPath {
    fn as_str(&self) -> &'static str {
        match self {
            // 没错，就是这么奇葩，获取用户信息也是用的/login
            // 带AVS去请求/login，就能获取用户信息，而不需要用户名密码
            // 如果AVS无效或过期，就走正常的登录流程
            ApiPath::Login | ApiPath::GetUserProfile => "/login",
            ApiPath::Search => "/search",
            ApiPath::GetComic => "/album",
            ApiPath::GetChapter => "/chapter",
            ApiPath::GetScrambleId => "/chapter_view_template",
            ApiPath::GetFavoriteFolder => "/favorite",
        }
    }
}

#[derive(Clone)]
pub struct JmClient {
    app: AppHandle,
    api_client: Arc<RwLock<ClientWithMiddleware>>,
    api_jar: Arc<Jar>,
    img_client: Arc<RwLock<ClientWithMiddleware>>,
}

impl JmClient {
    pub fn new(app: AppHandle) -> Self {
        let api_jar = Arc::new(Jar::default());
        let api_client = create_api_client(&app, &api_jar);
        let api_client = Arc::new(RwLock::new(api_client));

        let img_client = create_img_client(&app);
        let img_client = Arc::new(RwLock::new(img_client));

        Self {
            app,
            api_client,
            api_jar,
            img_client,
        }
    }

    pub fn reload_client(&self) {
        let api_client = create_api_client(&self.app, &self.api_jar);
        *self.api_client.write() = api_client;
        let img_client = create_img_client(&self.app);
        *self.img_client.write() = img_client;
    }

    async fn jm_request(
        &self,
        method: reqwest::Method,
        path: ApiPath,
        query: Option<serde_json::Value>,
        form: Option<serde_json::Value>,
        ts: u64,
    ) -> anyhow::Result<reqwest::Response> {
        let tokenparam = format!("{ts},{APP_VERSION}");
        let token = if path == ApiPath::GetScrambleId {
            utils::md5_hex(&format!("{ts}{APP_TOKEN_SECRET_2}"))
        } else {
            utils::md5_hex(&format!("{ts}{APP_TOKEN_SECRET}"))
        };

        let path = path.as_str();
        let request = self
            .api_client
            .read()
            .request(method, format!("https://{API_DOMAIN}{path}").as_str())
            .header("token", token)
            .header("tokenparam", tokenparam)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36");

        let http_resp = match form {
            Some(payload) => request.query(&query).form(&payload).send().await,
            None => request.query(&query).send().await,
        }
        .map_err(|e| {
            if e.is_timeout() {
                anyhow::Error::from(e).context("连接超时，请使用代理或换条线路重试")
            } else {
                anyhow::Error::from(e)
            }
        })?;

        Ok(http_resp)
    }

    async fn jm_get(
        &self,
        path: ApiPath,
        query: Option<serde_json::Value>,
        ts: u64,
    ) -> anyhow::Result<reqwest::Response> {
        self.jm_request(reqwest::Method::GET, path, query, None, ts)
            .await
    }

    async fn jm_post(
        &self,
        path: ApiPath,
        query: Option<serde_json::Value>,
        payload: Option<serde_json::Value>,
        ts: u64,
    ) -> anyhow::Result<reqwest::Response> {
        self.jm_request(reqwest::Method::POST, path, query, payload, ts)
            .await
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
    ) -> anyhow::Result<GetUserProfileRespData> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let form = json!({
            "username": username,
            "password": password,
        });
        // 发送登录请求
        let http_resp = self.jm_post(ApiPath::Login, None, Some(form), ts).await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!(
                "使用账号密码登录失败，预料之外的状态码({status}): {body}"
            ));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("使用账号密码登录失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp.data.as_str().context(format!(
            "使用账号密码登录失败，data字段不是字符串: {jm_resp:?}"
        ))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的data字段解析为GetUserProfileRespData
        let mut user_profile = serde_json::from_str::<GetUserProfileRespData>(&data).context(
            format!("将解密后的data字段解析为GetUserProfileRespData失败: {data}"),
        )?;
        user_profile.photo = format!("https://{IMAGE_DOMAIN}/media/users/{}", user_profile.photo);

        Ok(user_profile)
    }

    pub async fn get_user_profile(&self) -> anyhow::Result<GetUserProfileRespData> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        // 发送获取用户信息请求
        let http_resp = self
            .jm_post(ApiPath::GetUserProfile, None, None, ts)
            .await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(anyhow!("获取用户信息失败，Cookie无效或已过期，请重新登录"));
        } else if status != reqwest::StatusCode::OK {
            return Err(anyhow!(
                "获取用户信息失败，预料之外的状态码({status}): {body}"
            ));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("获取用户信息失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp
            .data
            .as_str()
            .context(format!("获取用户信息失败，data字段不是字符串: {jm_resp:?}"))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的data字段解析为GetUserProfileRespData
        let mut user_profile = serde_json::from_str::<GetUserProfileRespData>(&data).context(
            format!("将解密后的data字段解析为GetUserProfileRespData失败: {data}"),
        )?;
        user_profile.photo = format!("https://{IMAGE_DOMAIN}/media/users/{}", user_profile.photo);

        Ok(user_profile)
    }

    pub async fn search(
        &self,
        keyword: &str,
        page: i64,
        sort: SearchSort,
    ) -> anyhow::Result<SearchResp> {
        let query = json!({
            "main_tag": 0,
            "search_query": keyword,
            "page": page,
            "o": sort.as_str(),
        });
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        // 发送搜索请求
        let http_resp = self.jm_get(ApiPath::Search, Some(query), ts).await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!("搜索失败，预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("搜索失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp
            .data
            .as_str()
            .context(format!("搜索失败，data字段不是字符串: {jm_resp:?}"))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的数据解析为 RedirectRespData
        if let Ok(redirect_resp_data) = serde_json::from_str::<RedirectRespData>(&data) {
            let comic_resp_data = self
                .get_comic(redirect_resp_data.redirect_aid.parse()?)
                .await?;
            return Ok(SearchResp::ComicRespData(Box::new(comic_resp_data)));
        }
        // 尝试将解密后的data字段解析为 SearchRespData
        if let Ok(search_resp_data) = serde_json::from_str::<SearchRespData>(&data) {
            return Ok(SearchResp::SearchRespData(search_resp_data));
        }
        Err(anyhow!(
            "将解密后的数据解析为SearchRespData或RedirectRespData失败: {data}"
        ))
    }

    pub async fn get_comic(&self, aid: i64) -> anyhow::Result<GetComicRespData> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let query = json!({"id": aid,});
        // 发送获取漫画请求
        let http_resp = self.jm_get(ApiPath::GetComic, Some(query), ts).await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!("获取漫画失败，预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("获取漫画失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp
            .data
            .as_str()
            .context(format!("获取漫画失败，data字段不是字符串: {jm_resp:?}"))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的data字段解析为GetComicRespData
        let comic = serde_json::from_str::<GetComicRespData>(&data).context(format!(
            "将解密后的data字段解析为GetComicRespData失败: {data}"
        ))?;
        Ok(comic)
    }

    pub async fn get_chapter(&self, id: i64) -> anyhow::Result<GetChapterRespData> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let query = json!({"id": id,});
        // 发送获取章节请求
        let http_resp = self.jm_get(ApiPath::GetChapter, Some(query), ts).await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!("获取章节失败，预料之外的状态码({status}): {body}"));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("获取章节失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp
            .data
            .as_str()
            .context(format!("获取章节失败，data字段不是字符串: {jm_resp:?}"))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的data字段解析为GetChapterRespData
        let chapter = serde_json::from_str::<GetChapterRespData>(&data).context(format!(
            "将解密后的data字段解析为GetChapterRespData失败: {data}"
        ))?;
        Ok(chapter)
    }

    pub async fn get_scramble_id(&self, id: i64) -> anyhow::Result<i64> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let query = json!({
            "id": id,
            "v": ts,
            "mode": "vertical",
            "page": 0,
            "app_img_shunt": 1,
            "express": "off",
        });
        // 发送获取scramble_id请求
        let http_resp = self.jm_get(ApiPath::GetScrambleId, Some(query), ts).await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!(
                "获取scramble_id失败，预料之外的状态码({status}): {body}"
            ));
        }
        // 从body中提取scramble_id，如果提取失败则使用默认值
        let scramble_id = body
            .split("var scramble_id = ")
            .nth(1)
            .and_then(|s| s.split(';').next())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(220_980);
        Ok(scramble_id)
    }

    pub async fn get_favorite_folder(
        &self,
        folder_id: i64,
        page: i64,
        sort: FavoriteSort,
    ) -> anyhow::Result<GetFavoriteRespData> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let query = json!({
            "page": page,
            "o": sort.as_str(),
            "folder_id": folder_id,
        });
        // 发送获取收藏夹请求
        let http_resp = self
            .jm_get(ApiPath::GetFavoriteFolder, Some(query), ts)
            .await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!(
                "获取收藏夹失败，预料之外的状态码({status}): {body}"
            ));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("获取收藏夹失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp
            .data
            .as_str()
            .context(format!("获取收藏夹失败，data字段不是字符串: {jm_resp:?}"))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的data字段解析为GetFavoriteRespData
        let favorite = serde_json::from_str::<GetFavoriteRespData>(&data).context(format!(
            "将解密后的data字段解析为GetFavoriteRespData失败: {data}"
        ))?;
        Ok(favorite)
    }

    pub async fn toggle_favorite_comic(&self, aid: i64) -> anyhow::Result<ToggleFavoriteRespData> {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let form = json!({
            "aid": aid,
        });
        // 发送 收藏/取消收藏 请求
        let http_resp = self
            .jm_post(ApiPath::GetFavoriteFolder, None, Some(form), ts)
            .await?;
        // 检查http响应状态码
        let status = http_resp.status();
        let body = http_resp.text().await?;
        if status != reqwest::StatusCode::OK {
            return Err(anyhow!(
                "收藏/取消收藏 失败，预料之外的状态码({status}): {body}"
            ));
        }
        // 尝试将body解析为JmResp
        let jm_resp = serde_json::from_str::<JmResp>(&body)
            .context(format!("将body解析为JmResp失败: {body}"))?;
        // 检查JmResp的code字段
        if jm_resp.code != 200 {
            return Err(anyhow!("收藏/取消收藏 失败，预料之外的code: {jm_resp:?}"));
        }
        // 检查JmResp的data字段
        let data = jm_resp.data.as_str().context(format!(
            "收藏/取消收藏 失败，data字段不是字符串: {jm_resp:?}"
        ))?;
        // 解密data字段
        let data = decrypt_data(ts, data)?;
        // 尝试将解密后的data字段解析为ToggleFavoriteRespData
        let toggle_favorite_resp_data = serde_json::from_str::<ToggleFavoriteRespData>(&data)
            .context(format!(
                "将解密后的data字段解析为ToggleFavoriteRespData失败: {data}"
            ))?;
        Ok(toggle_favorite_resp_data)
    }

    pub async fn get_img_data(&self, url: &str) -> anyhow::Result<Bytes> {
        let request = self.img_client.read().get(url);

        let http_resp = request.send().await?;
        let status = http_resp.status();
        if status != StatusCode::OK {
            let text = http_resp.text().await?;
            let err = anyhow!("下载图片`{url}`失败，预料之外的状态码: {text}");
            return Err(err);
        }

        let mut image_data = http_resp.bytes().await?;

        if image_data.is_empty() {
            // 如果图片为空，说明jm那边缓存失效了，带上时间戳再次请求，以避免缓存
            let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            let query = json!({"ts": ts});
            let request = self.img_client.read().get(url).query(&query);

            let http_resp = request.send().await?;
            let status = http_resp.status();
            if status != StatusCode::OK {
                let text = http_resp.text().await?;
                let err = anyhow!("下载图片`{url}`失败，预料之外的状态码: {text}");
                return Err(err);
            }

            image_data = http_resp.bytes().await?;
        }

        Ok(image_data)
    }
}

pub fn create_api_client(app: &AppHandle, jar: &Arc<Jar>) -> ClientWithMiddleware {
    let builder = reqwest::ClientBuilder::new().cookie_provider(jar.clone());

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
                    let err_title = format!("`JmClient`设置代理`{proxy_url}`失败");
                    let string_chain = err.to_string_chain();
                    tracing::error!(err_title, message = string_chain);
                    builder
                }
            }
        }
    };

    let retry_policy = ExponentialBackoff::builder()
        .base(1) // 指数为1，保证重试间隔为1秒不变
        .jitter(Jitter::Bounded) // 重试间隔在1秒左右波动
        .build_with_total_retry_duration(Duration::from_secs(5)); // 重试总时长为5秒

    reqwest_middleware::ClientBuilder::new(builder.build().unwrap())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

pub fn create_img_client(app: &AppHandle) -> ClientWithMiddleware {
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

fn decrypt_data(ts: u64, data: &str) -> anyhow::Result<String> {
    // 使用Base64解码传入的数据，得到AES-256-ECB加密的数据
    let aes256_ecb_encrypted_data = general_purpose::STANDARD.decode(data)?;
    // 生成密钥
    let key = utils::md5_hex(&format!("{ts}{APP_DATA_SECRET}"));
    // 使用AES-256-ECB进行解密
    let cipher = Aes256::new(GenericArray::from_slice(key.as_bytes()));
    let decrypted_data_with_padding: Vec<u8> = aes256_ecb_encrypted_data
        .chunks(16)
        .map(GenericArray::clone_from_slice)
        .flat_map(|mut block| {
            cipher.decrypt_block(&mut block);
            block.to_vec()
        })
        .collect();
    // 去除PKCS#7填充，根据最后一个字节的值确定填充长度
    let padding_length = decrypted_data_with_padding.last().copied().unwrap() as usize;
    let decrypted_data_without_padding =
        decrypted_data_with_padding[..decrypted_data_with_padding.len() - padding_length].to_vec();
    // 将解密后的数据转换为UTF-8字符串
    let decrypted_data = String::from_utf8(decrypted_data_without_padding)?;
    Ok(decrypted_data)
}
