import { platform } from "@tauri-apps/plugin-os";
import { useKeyboard, Key } from "@waradu/keyboard";

export default defineNuxtPlugin(async (nuxtApp) => {
  const keyboardInstance = useKeyboard();
  let currentOS = "windows";
  try {
    const osName = await Promise.resolve(platform());
    currentOS = osName.toLowerCase().includes("mac") ? "macos" : "windows";
  } catch (error) {
    console.error("Error detecting platform:", error);
  }

  // Defer initialization until the app is mounted
  nuxtApp.hook('app:mounted', () => {
    keyboardInstance.init();
  });

  nuxtApp.provide('keyboard', {
    listen: keyboardInstance.listen.bind(keyboardInstance),
    init: keyboardInstance.init.bind(keyboardInstance),
    Key,
    currentOS,
    // Provide a clear method if users need to manually clear all listeners from the instance
    clearAll: keyboardInstance.clear ? keyboardInstance.clear.bind(keyboardInstance) : () => { console.warn('@waradu/keyboard instance does not have a clear method'); }
  });
});
