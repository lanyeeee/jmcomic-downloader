use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum SearchSort {
    Latest,
    View,
    Picture,
    Like,
}

impl SearchSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchSort::Latest => "mr",
            SearchSort::View => "mv",
            SearchSort::Picture => "mp",
            SearchSort::Like => "tf",
        }
    }
}
