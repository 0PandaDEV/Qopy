import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "~/types/types";

export default defineNuxtPlugin(() => {
  return {
    provide: {
      settings: {
        async getSetting(key: string): Promise<string> {
          return await invoke<string>("get_setting", { key });
        },

        async saveSetting(key: string, value: string): Promise<void> {
          await invoke<void>("save_setting", { key, value });
        },
      },
    },
  };
});
