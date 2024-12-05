export interface HistoryItem {
  id: string;
  content_type: ContentType;
  content: string;
  favicon: string;
  timestamp: Date;
}

export interface Settings {
  key: string;
  value: string;
}

export enum ContentType {
  TEXT,
  IMAGE,
  FILE,
  LINK,
  COLOR,
  CODE,
}
