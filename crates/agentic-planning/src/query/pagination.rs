//! Cursor-based pagination.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub cursor: Option<String>,
    pub has_more: bool,
    pub total: Option<usize>,
}

impl<T> CursorPage<T> {
    pub fn empty() -> Self {
        Self { items: Vec::new(), cursor: None, has_more: false, total: Some(0) }
    }

    pub fn from_slice(all: Vec<T>, cursor: Option<&str>, limit: usize) -> Self
    where T: Clone
    {
        let start = cursor.and_then(|c| c.parse::<usize>().ok()).unwrap_or(0);
        let total = all.len();

        if start >= total {
            return Self { items: Vec::new(), cursor: None, has_more: false, total: Some(total) };
        }

        let end = (start + limit).min(total);
        let items = all[start..end].to_vec();
        let has_more = end < total;
        let next_cursor = if has_more { Some(end.to_string()) } else { None };

        Self { items, cursor: next_cursor, has_more, total: Some(total) }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
