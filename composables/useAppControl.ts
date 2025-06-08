import { app, window } from "@tauri-apps/api";

export function useAppControl() {
  const hideApp = async (): Promise<void> => {
    await app.hide();
    await window.getCurrentWindow().hide();
  };

  return {
    hideApp
  };
} 