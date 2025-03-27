use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum FavoriteSort {
    FavoriteTime,
    UpdateTime,
}

impl FavoriteSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            FavoriteSort::FavoriteTime => "mr",
            FavoriteSort::UpdateTime => "mp",
        }
    }
}
