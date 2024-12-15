use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct HistoryItem {
    pub id: String,
    pub source: String,
    pub source_icon: Option<String>,
    pub content_type: ContentType,
    pub content: String,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Text {
    pub source: String,
    pub content_type: ContentType,
    pub characters: i32,
    pub words: i32,
    pub copied: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Image {
    pub source: String,
    pub content_type: ContentType,
    pub dimensions: String,
    pub size: i64,
    pub copied: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    pub source: String,
    pub content_type: ContentType,
    pub path: String,
    pub filesize: i64,
    pub copied: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Link {
    pub source: String,
    pub content_type: ContentType,
    pub title: String,
    pub link: String,
    pub characters: i32,
    pub copied: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Color {
    pub source: String,
    pub content_type: ContentType,
    pub hexcode: String,
    pub rgba: String,
    pub copied: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Code {
    pub source: String,
    pub content_type: ContentType,
    pub language: String,
    pub lines: i32,
    pub copied: DateTime<Utc>,
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
    pub fn new(source: String, content_type: ContentType, content: String, favicon: Option<String>, source_icon: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source,
            source_icon,
            content_type,
            content,
            favicon,
            timestamp: Utc::now(),
        }
    }

    pub fn to_row(&self) -> (String, String, Option<String>, String, String, Option<String>, DateTime<Utc>) {
        (
            self.id.clone(),
            self.source.clone(),
            self.source_icon.clone(),
            self.content_type.to_string(),
            self.content.clone(),
            self.favicon.clone(),
            self.timestamp,
        )
    }
}
