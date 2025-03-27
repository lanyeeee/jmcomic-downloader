mod archive_format;
mod chapter_info;
mod comic;
mod download_format;
mod favorite_sort;
mod proxy_mode;
mod search_result;
mod search_sort;

pub use archive_format::*;
pub use chapter_info::*;
pub use comic::*;
pub use download_format::*;
pub use favorite_sort::*;
pub use proxy_mode::*;
pub use search_result::*;
pub use search_sort::*;

pub type AsyncRwLock<T> = tokio::sync::RwLock<T>;
