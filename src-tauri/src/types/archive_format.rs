use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum ArchiveFormat {
    #[default]
    Image,
    Pdf,
}

impl ArchiveFormat {
    pub fn extension(&self) -> &str {
        match self {
            ArchiveFormat::Image => "",
            ArchiveFormat::Pdf => "pdf",
        }
    }
}
