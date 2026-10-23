use std::path::{Path, PathBuf};

use crate::types::{DownloadFormat, ProxyMode};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

const API_DOMAIN_1: &str = "www.cdnzack.cc";
const API_DOMAIN_2: &str = "www.cdnhth.cc";
const API_DOMAIN_3: &str = "www.cdnhth.net";
const API_DOMAIN_4: &str = "www.cdnbea.net";
const API_DOMAIN_5: &str = "www.cdn-mspjmapiproxy.xyz";

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub username: String,
    pub password: String,
    pub download_dir: PathBuf,
    pub export_dir: PathBuf,
    pub download_format: DownloadFormat,
    pub dir_fmt: String,
    pub proxy_mode: ProxyMode,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub enable_file_logger: bool,
    pub chapter_concurrency: usize,
    pub chapter_download_interval_sec: u64,
    pub img_concurrency: usize,
    pub img_download_interval_sec: u64,
    pub download_all_favorites_interval_sec: u64,
    pub update_downloaded_comics_interval_sec: u64,
    pub api_domain_mode: ApiDomainMode,
    pub custom_api_domain: String,
    pub should_download_cover: bool,
    pub create_pdf_concurrency: usize,
    pub enable_merge_pdf: bool,
    /// 导出跳过模式
    pub export_skip_mode: ExportSkipMode,
}

impl Config {
    pub fn new(app: &AppHandle) -> eyre::Result<Self> {
        let app_data_dir = app.path().app_data_dir()?;
        let config_path = app_data_dir.join("config.json");

        let config = if config_path.exists() {
            let config_string = std::fs::read_to_string(config_path)?;
            match serde_json::from_str(&config_string) {
                // 如果能够直接解析为Config，则直接返回
                Ok(config) => config,
                // 否则，将默认配置与文件中已有的配置合并
                // 以免新版本添加了新的配置项，用户升级到新版本后，所有配置项都被重置
                Err(_) => Config::merge_config(&config_string, &app_data_dir),
            }
        } else {
            Config::default(&app_data_dir)
        };
        config.save(app)?;
        Ok(config)
    }

    pub fn save(&self, app: &AppHandle) -> eyre::Result<()> {
        let resource_dir = app.path().app_data_dir()?;
        let config_path = resource_dir.join("config.json");
        let config_string = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, config_string)?;
        Ok(())
    }

    pub fn get_api_domain(&self) -> String {
        match self.api_domain_mode {
            ApiDomainMode::Domain1 => API_DOMAIN_1.to_string(),
            ApiDomainMode::Domain2 => API_DOMAIN_2.to_string(),
            ApiDomainMode::Domain3 => API_DOMAIN_3.to_string(),
            ApiDomainMode::Domain4 => API_DOMAIN_4.to_string(),
            ApiDomainMode::Domain5 => API_DOMAIN_5.to_string(),
            ApiDomainMode::Custom => self.custom_api_domain.clone(),
        }
    }

    pub fn get_api_domains(&self) -> Vec<String> {
        let mut domains = vec![self.get_api_domain()];
        if self.api_domain_mode != ApiDomainMode::Custom {
            for domain in [
                API_DOMAIN_1,
                API_DOMAIN_2,
                API_DOMAIN_3,
                API_DOMAIN_4,
                API_DOMAIN_5,
            ] {
                if domains[0] != domain {
                    domains.push(domain.to_string());
                }
            }
        }
        domains
    }

    fn merge_config(config_string: &str, app_data_dir: &Path) -> Config {
        let Ok(mut json_value) = serde_json::from_str::<serde_json::Value>(config_string) else {
            return Config::default(app_data_dir);
        };
        let serde_json::Value::Object(ref mut map) = json_value else {
            return Config::default(app_data_dir);
        };
        let Ok(default_config_value) = serde_json::to_value(Config::default(app_data_dir)) else {
            return Config::default(app_data_dir);
        };
        let serde_json::Value::Object(default_map) = default_config_value else {
            return Config::default(app_data_dir);
        };
        for (key, value) in default_map {
            map.entry(key).or_insert(value);
        }
        let Ok(config) = serde_json::from_value(json_value) else {
            return Config::default(app_data_dir);
        };
        config
    }

    fn default(app_data_dir: &Path) -> Config {
        let cpu_core_num = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1);

        Config {
            username: String::new(),
            password: String::new(),
            download_dir: app_data_dir.join("漫画下载"),
            export_dir: app_data_dir.join("漫画导出"),
            download_format: DownloadFormat::default(),
            dir_fmt: "{comic_title}/{chapter_title}".to_string(),
            proxy_mode: ProxyMode::default(),
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 7890,
            enable_file_logger: true,
            chapter_concurrency: 3,
            chapter_download_interval_sec: 0,
            img_concurrency: 20,
            img_download_interval_sec: 0,
            download_all_favorites_interval_sec: 0,
            update_downloaded_comics_interval_sec: 0,
            api_domain_mode: ApiDomainMode::Domain2,
            custom_api_domain: API_DOMAIN_2.to_string(),
            should_download_cover: true,
            create_pdf_concurrency: cpu_core_num,
            enable_merge_pdf: true,
            export_skip_mode: ExportSkipMode::default(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum ApiDomainMode {
    Domain1,
    #[default]
    Domain2,
    Domain3,
    Domain4,
    Domain5,
    Custom,
}

/// 导出跳过模式
#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum ExportSkipMode {
    /// 每次重新导出所有章节
    #[default]
    None,
    /// 跳过本地已存在的导出文件
    SkipExisting,
    /// 跳过曾导出过的章节（即使本地文件已删除）
    SkipExported,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_domains_start_with_manual_selection_and_appear_once() {
        for (mode, selected) in [
            (ApiDomainMode::Domain1, API_DOMAIN_1),
            (ApiDomainMode::Domain2, API_DOMAIN_2),
            (ApiDomainMode::Domain3, API_DOMAIN_3),
            (ApiDomainMode::Domain4, API_DOMAIN_4),
            (ApiDomainMode::Domain5, API_DOMAIN_5),
        ] {
            let mut config = Config::default(Path::new("."));
            config.api_domain_mode = mode;
            let domains = config.get_api_domains();
            assert_eq!(domains[0], selected);
            assert_eq!(domains.len(), 5);
            for domain in [
                API_DOMAIN_1,
                API_DOMAIN_2,
                API_DOMAIN_3,
                API_DOMAIN_4,
                API_DOMAIN_5,
            ] {
                assert_eq!(
                    domains
                        .iter()
                        .filter(|candidate| *candidate == domain)
                        .count(),
                    1
                );
            }
        }
    }

    #[test]
    fn custom_domain_does_not_add_built_in_fallbacks() {
        let mut config = Config::default(Path::new("."));
        config.api_domain_mode = ApiDomainMode::Custom;
        config.custom_api_domain = "api.example.test".to_string();
        assert_eq!(config.get_api_domains(), vec!["api.example.test"]);
    }
}
