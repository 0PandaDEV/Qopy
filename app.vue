<template>
  <div style="pointer-events: auto">
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import { app, window } from "@tauri-apps/api";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import { onMounted } from "vue";
import { keyboard } from "wrdu-keyboard";

const { $settings } = useNuxtApp();
keyboard.init()

onMounted(async () => {
  await listen("settings", async () => {
    await navigateTo("/settings");
    await app.show();
    await window.getCurrentWindow().show();
  });

  if ((await $settings.getSetting("autostart")) === "true") {
    await enable();
  } else {
    await disable();
  }

  await listen("main_route", async () => {
    await navigateTo("/");
  });
});
</script>

<style lang="scss">
@font-face {
  font-family: SFRoundedRegular;
  font-display: swap;
  src: url("~/assets/fonts/SFRoundedRegular.otf") format("opentype");
}

@font-face {
  font-family: SFRoundedMedium;
  font-display: swap;
  src: url("~/assets/fonts/SFRoundedMedium.otf") format("opentype");
}

@font-face {
  font-family: SFRoundedSemiBold;
  font-display: swap;
  src: url("~/assets/fonts/SFRoundedSemiBold.otf") format("opentype");
}

@font-face {
  font-family: CommitMono;
  font-display: swap;
  src: url("~/assets/fonts/CommitMono.woff2") format("woff2");
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  text-decoration: none;
  font-family: SFRoundedRegular;
  scroll-behavior: smooth;
  scrollbar-width: thin;
  user-select: none;
  position: relative;
  z-index: 1;

  --os-handle-bg: #ada9a1;
  --os-handle-bg-hover: #78756f;
  --os-handle-bg-active: #78756f;
}

html,
body {
  background-color: transparent;
  width: 750px;
  height: 474px;
  user-select: none !important;
  pointer-events: none !important;
}

.os-scrollbar-horizontal {
  display: none;
}
</style>
