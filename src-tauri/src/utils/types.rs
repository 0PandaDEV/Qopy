use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HistoryItem {
    pub id: String,
    pub content_type: ContentType,
    pub content: String,
    #[serde(default)]
    pub favicon: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text,
    Image,
    File,
    Link,
    Color,
    Code,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::Text => write!(f, "text"),
            ContentType::Image => write!(f, "image"),
            ContentType::File => write!(f, "file"),
            ContentType::Link => write!(f, "link"),
            ContentType::Color => write!(f, "color"),
            ContentType::Code => write!(f, "code"),
        }
    }
}

impl From<String> for ContentType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "text" => ContentType::Text,
            "image" => ContentType::Image,
            "file" => ContentType::File,
            "link" => ContentType::Link,
            "color" => ContentType::Color,
            "code" => ContentType::Code,
            _ => ContentType::Text,
        }
    }
}

impl HistoryItem {
    pub fn new(content_type: ContentType, content: String, favicon: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            content_type,
            content,
            favicon,
            timestamp: Utc::now(),
        }
    }

    pub fn to_row(&self) -> (String, String, String, Option<String>, DateTime<Utc>) {
        (
            self.id.clone(),
            self.content_type.to_string(),
            self.content.clone(),
            self.favicon.clone(),
            self.timestamp,
        )
    }
}
