use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum DownloadFormat {
    #[default]
    Jpeg,
    Png,
    Webp,
}

impl DownloadFormat {
    pub fn extension(self) -> &'static str {
        match self {
            DownloadFormat::Jpeg => "jpg",
            DownloadFormat::Png => "png",
            DownloadFormat::Webp => "webp",
        }
    }
}
