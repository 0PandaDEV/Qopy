import { v4 as uuidv4 } from 'uuid';

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
  content_type: ContentType;
  content: string;
  favicon?: string;
  timestamp: Date;

  constructor(content_type: ContentType, content: string, favicon?: string) {
    this.id = uuidv4();
    this.content_type = content_type;
    this.content = content;
    this.favicon = favicon;
    this.timestamp = new Date();
  }

  toRow(): [string, string, string, string | undefined, Date] {
    return [this.id, this.content_type, this.content, this.favicon, this.timestamp];
  }
}

export interface Settings {
  key: string;
  value: string;
}
