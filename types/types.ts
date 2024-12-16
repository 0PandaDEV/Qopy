import { v4 as uuidv4 } from "uuid";

export enum ContentType {
  Text = "text",
  Image = "image",
  File = "file",
  Link = "link",
  Color = "color",
  Code = "code",
}

export class HistoryItem {
  id: string;
  source: string;
  source_icon?: string;
  content_type: ContentType;
  content: string;
  favicon?: string;
  timestamp: Date;
  language?: string;

  constructor(
    source: string,
    content_type: ContentType,
    content: string,
    favicon?: string,
    source_icon?: string,
    language?: string
  ) {
    this.id = uuidv4();
    this.source = source;
    this.source_icon = source_icon;
    this.content_type = content_type;
    this.content = content;
    this.favicon = favicon;
    this.timestamp = new Date();
    this.language = language;
  }

  toRow(): [
    string,
    string,
    string | undefined,
    string,
    string,
    string | undefined,
    Date,
    string | undefined
  ] {
    return [
      this.id,
      this.source,
      this.source_icon,
      this.content_type,
      this.content,
      this.favicon,
      this.timestamp,
      this.language,
    ];
  }
}

export interface Settings {
  key: string;
  value: string;
}

export interface Text {
  source: string;
  content_type: ContentType.Text;
  characters: number;
  words: number;
  copied: Date;
}

export interface Image {
  source: string;
  content_type: ContentType.Image;
  dimensions: string;
  size: number;
  copied: Date;
}

export interface File {
  source: string;
  content_type: ContentType.File;
  path: string;
  filesize: number;
  copied: Date;
}

export interface Link {
  source: string;
  content_type: ContentType.Link;
  title: string;
  link: string;
  characters: number;
  copied: Date;
}

export interface Color {
  source: string;
  content_type: ContentType.Color;
  hexcode: string;
  rgba: string;
  copied: Date;
}

export interface Code {
  source: string;
  content_type: ContentType.Code;
  language: string;
  lines: number;
  copied: Date;
}
