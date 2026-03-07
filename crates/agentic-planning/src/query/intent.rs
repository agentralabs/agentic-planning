//! Intent declaration and scoped extraction.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExtractionIntent {
    Exists,
    #[default]
    IdsOnly,
    Summary,
    Fields(Vec<String>),
    Full,
}

impl ExtractionIntent {
    pub fn estimated_tokens(&self) -> u64 {
        match self {
            Self::Exists => 1,
            Self::IdsOnly => 10,
            Self::Summary => 50,
            Self::Fields(f) => 20 * f.len() as u64,
            Self::Full => 500,
        }
    }

    pub fn parse_label(s: &str) -> Self {
        match s {
            "exists" => Self::Exists,
            "ids" | "ids_only" => Self::IdsOnly,
            "summary" => Self::Summary,
            "full" => Self::Full,
            _ => Self::Full,
        }
    }

    pub fn is_minimal(&self) -> bool {
        matches!(self, Self::Exists | Self::IdsOnly)
    }

    pub fn includes_content(&self) -> bool {
        matches!(self, Self::Summary | Self::Fields(_) | Self::Full)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopedResult {
    Bool(bool),
    Id(String),
    Ids(Vec<String>),
    Summary(String),
    Fields(HashMap<String, Value>),
    Full(Value),
    Count(usize),
}

impl ScopedResult {
    pub fn estimated_tokens(&self) -> u64 {
        match self {
            Self::Bool(_) => 1,
            Self::Id(_) => 5,
            Self::Ids(ids) => ids.len() as u64 * 5,
            Self::Summary(_) => 50,
            Self::Fields(f) => f.len() as u64 * 20,
            Self::Full(v) => serde_json::to_string(v)
                .map(|s| s.len() as u64 / 4)
                .unwrap_or(500),
            Self::Count(_) => 2,
        }
    }
}

pub trait Scopeable {
    fn id_str(&self) -> String;
    fn summarize(&self) -> String;
    fn extract_fields(&self, fields: &[String]) -> HashMap<String, Value>;
    fn to_json(&self) -> Value;
}

pub fn apply_intent<T: Scopeable>(intent: &ExtractionIntent, item: &T) -> ScopedResult {
    match intent {
        ExtractionIntent::Exists => ScopedResult::Bool(true),
        ExtractionIntent::IdsOnly => ScopedResult::Id(item.id_str()),
        ExtractionIntent::Summary => ScopedResult::Summary(item.summarize()),
        ExtractionIntent::Fields(f) => ScopedResult::Fields(item.extract_fields(f)),
        ExtractionIntent::Full => ScopedResult::Full(item.to_json()),
    }
}

pub fn apply_intent_many<T: Scopeable>(intent: &ExtractionIntent, items: &[T]) -> ScopedResult {
    match intent {
        ExtractionIntent::Exists => ScopedResult::Bool(!items.is_empty()),
        ExtractionIntent::IdsOnly => ScopedResult::Ids(items.iter().map(|i| i.id_str()).collect()),
        ExtractionIntent::Summary => ScopedResult::Count(items.len()),
        _ => ScopedResult::Full(serde_json::json!(items
            .iter()
            .map(|i| i.to_json())
            .collect::<Vec<_>>())),
    }
}
