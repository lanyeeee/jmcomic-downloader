use std::{collections::HashMap, path::PathBuf};

use anyhow::Context;
use parking_lot::RwLock;
use tauri::{AppHandle, Manager};
use walkdir::WalkDir;

use crate::{config::Config, extensions::WalkDirEntryExt};

pub fn filename_filter(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\\' | '/' | '\n' => ' ',
            ':' => '：',
            '*' => '⭐',
            '?' => '？',
            '"' => '\'',
            '<' => '《',
            '>' => '》',
            '|' => '丨',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .trim_end_matches('.')
        .trim()
        .to_string()
}

// 计算MD5哈希并返回十六进制字符串
pub fn md5_hex(data: &str) -> String {
    format!("{:x}", md5::compute(data))
}

pub fn create_id_to_dir_map(app: &AppHandle) -> anyhow::Result<HashMap<i64, PathBuf>> {
    let mut id_to_dir_map: HashMap<i64, PathBuf> = HashMap::new();
    let download_dir = app.state::<RwLock<Config>>().read().download_dir.clone();
    if !download_dir.exists() {
        return Ok(id_to_dir_map);
    }

    for entry in WalkDir::new(&download_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if !entry.is_comic_metadata() {
            continue;
        }

        let metadata_str =
            std::fs::read_to_string(path).context(format!("读取`{}`失败", path.display()))?;
        let comic_json: serde_json::Value = serde_json::from_str(&metadata_str).context(
            format!("将`{}`反序列化为serde_json::Value失败", path.display()),
        )?;
        let id = comic_json
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .context(format!("`{}`没有`id`字段", path.display()))?;

        let parent = path
            .parent()
            .context(format!("`{}`没有父目录", path.display()))?;

        id_to_dir_map.entry(id).or_insert(parent.to_path_buf());
    }
    Ok(id_to_dir_map)
}
