use serde::{Deserialize, Serialize};

pub const DEFAULT_PAGE_SIZE: &'static u8 = &25;
pub const MAX_PAGE_SIZE: &'static u8 = &100;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pagination {
    pub has_more: bool,
    pub has_less: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}
