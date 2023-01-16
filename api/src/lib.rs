use serde::{Deserialize, Serialize};

pub mod fighter;
pub mod tournament;
pub mod tournament_detail;
pub mod util;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub total_count: u64,
    pub total_pages: u64,
    pub has_next_page: bool,
    pub current_page: u64,
    pub item_count: u64,
}
