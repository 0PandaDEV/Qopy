#[derive(Deserialize, Serialize)]
struct HistoryItem {
    id: String,
    content_type: ContentType,
    content: String,
    favicon: String,
    timestamp: DATETIME,
}

#[derive(Deserialize, Serialize)]
struct Settings {
    key: String,
    value: String,
}

enum ContentType {
    TEXT,
    IMAGE,
    FILE,
    LINK,
    COLOR,
    CODE,
}
