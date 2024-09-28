use std::fmt::Display;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use aes::Aes256;
use aes::cipher::{BlockDecrypt, KeyInit};
use aes::cipher::generic_array::GenericArray;
use anyhow::{anyhow, Context};
use base64::Engine;
use base64::engine::general_purpose;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::{Jitter, RetryTransientMiddleware};
use serde_json::json;
use tauri::{AppHandle, Manager};

use crate::config::Config;
use crate::extensions::IgnoreRwLockPoison;
use crate::responses::{JmResp, UserProfileRespData};

const APP_TOKEN_SECRET: &str = "18comicAPP";
const APP_TOKEN_SECRET_2: &str = "18comicAPPContent";
const APP_DATA_SECRET: &str = "185Hcomic3PAPP7R";
const APP_VERSION: &str = "1.7.3";

const API_DOMAIN: &str = "www.jmeadpoolcdn.life";
const IMAGE_DOMAIN: &str = "cdn-msp2.jmapiproxy2.cc";

#[derive(Debug, Clone, PartialEq)]
enum ApiPath {
    Login,
}
impl ApiPath {
    fn as_str(&self) -> &'static str {
        match self {
            ApiPath::Login => "/login",
        }
    }
}

#[derive(Clone)]
pub struct JmClient {
    app: AppHandle,
}

impl JmClient {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn client() -> ClientWithMiddleware {
        // TODO: 可以将retry_policy缓存起来，避免每次请求都创建
        let retry_policy = reqwest_retry::policies::ExponentialBackoff::builder()
            .base(1) // 指数为1，保证重试间隔为1秒不变
            .jitter(Jitter::Bounded) // 重试间隔在1秒左右波动
            .build_with_total_retry_duration(Duration::from_secs(3)); // 重试总时长为3秒
        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(2)) // 每个请求超过2秒就超时
            .build()
            .unwrap();
        reqwest_middleware::ClientBuilder::new(client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build()
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
        let token = md5_hex(&format!("{ts}{APP_TOKEN_SECRET}")); //TODO: 后面对于/chapter_view_template这个接口的请求，token的计算方式不一样
        let cookie = if path == ApiPath::Login && form.is_some() {
            String::new()
        } else {
            self.app
                .state::<RwLock<Config>>()
                .read_or_panic()
                .get_cookie()
        };

        let path = path.as_str();
        let request = Self::client()
            .request(method, format!("https://{API_DOMAIN}{path}").as_str())
            .header("token", token)
            .header("tokenparam", tokenparam)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .header("cookie", cookie);

        let http_resp = match form {
            Some(payload) => request.query(&query).form(&payload).send().await?,
            None => request.query(&query).send().await?,
        };

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
    ) -> anyhow::Result<UserProfileRespData> {
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
        // 尝试将解密后的data字段解析为UserProfile
        let user_profile = serde_json::from_str::<UserProfileRespData>(&data)
            .context(format!("将解密后的data字段解析为UserProfile失败: {data}"))?;

        Ok(user_profile)
    }
}

// 计算MD5哈希并返回十六进制字符串
fn md5_hex(data: &str) -> String {
    format!("{:x}", md5::compute(data))
}

fn decrypt_data(ts: u64, data: &str) -> anyhow::Result<String> {
    // 使用Base64解码传入的数据，得到AES-256-ECB加密的数据
    let aes256_ecb_encrypted_data = general_purpose::STANDARD.decode(data)?;
    // 生成密钥
    let key = md5_hex(&format!("{}{}", ts, APP_DATA_SECRET));
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

fn calculate_block_num(scramble_id: i64, id: i64, filename: &str) -> i64 {
    return if id < scramble_id {
        0
    } else if id < 268850 {
        10
    } else {
        let x = if id < 421926 { 10 } else { 8 };
        let s = format!("{}{}", id, filename);
        let s = md5_hex(&s);
        let mut block_num = s.chars().last().unwrap() as i64;
        block_num %= x;
        block_num = block_num * 2 + 2;
        block_num
    };
}
