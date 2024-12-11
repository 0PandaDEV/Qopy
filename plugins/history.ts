import { invoke } from "@tauri-apps/api/core";
import type { HistoryItem } from "~/types/types";

export default defineNuxtPlugin(() => {
  return {
    provide: {
      history: {
        async getHistory(): Promise<HistoryItem[]> {
          return await invoke<HistoryItem[]>("get_history");
        },

        async addHistoryItem(item: HistoryItem): Promise<void> {
          await invoke<void>("add_history_item", { item });
        },

        async searchHistory(query: string): Promise<HistoryItem[]> {
          return await invoke<HistoryItem[]>("search_history", { query });
        },

        async loadHistoryChunk(
          offset: number,
          limit: number
        ): Promise<HistoryItem[]> {
          return await invoke<HistoryItem[]>("load_history_chunk", {
            offset,
            limit,
          });
        },

        async getImagePath(path: string): Promise<string> {
          return await invoke<string>("get_image_path", { path });
        },

        async writeAndPaste(data: {
          content: string;
          contentType: string;
        }): Promise<void> {
          await invoke<void>("write_and_paste", data);
        },

        async readImage(data: { filename: string }): Promise<string> {
          return await invoke<string>("read_image", data);
        },
      },
    },
  };
});
