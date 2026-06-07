use serde::{Deserialize, Serialize};

/// Generic paginated response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items:       Vec<T>,
    pub total:       u64,
    pub page:        u32,
    pub page_size:   u32,
    pub total_pages: u32,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, page_size: u32) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            ((total as f64) / (page_size as f64)).ceil() as u32
        };
        Self { items, total, page, page_size, total_pages }
    }

    pub fn empty() -> Self {
        Self { items: vec![], total: 0, page: 1, page_size: 20, total_pages: 0 }
    }
}

/// Pagination input parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// 1-indexed
    pub page:      u32,
    pub page_size: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { page: 1, page_size: 20 }
    }
}

impl Pagination {
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.page_size
    }

    pub fn limit(&self) -> u32 {
        self.page_size.min(100) // hard cap at 100 per page
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_offset_first_page() {
        let p = Pagination { page: 1, page_size: 20 };
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn pagination_offset_second_page() {
        let p = Pagination { page: 2, page_size: 20 };
        assert_eq!(p.offset(), 20);
    }

    #[test]
    fn page_size_capped_at_100() {
        let p = Pagination { page: 1, page_size: 9999 };
        assert_eq!(p.limit(), 100);
    }

    #[test]
    fn page_new_calculates_total_pages() {
        let page: Page<i32> = Page::new(vec![1, 2, 3], 45, 1, 20);
        assert_eq!(page.total_pages, 3);
    }
}
