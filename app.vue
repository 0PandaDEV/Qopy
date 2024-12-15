<template>
  <div>
    <NuxtPage />
  </div>
</template>

<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { app, window } from '@tauri-apps/api';
import { onMounted } from 'vue'

onMounted(async () => {
  await listen('change_keybind', async () => {
    console.log("change_keybind");
    await navigateTo('/settings')
    await app.show();
    await window.getCurrentWindow().show();
  })

  await listen('main_route', async () => {
    console.log("main_route");
    await navigateTo('/')
  })
})
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
  color: #E5DFD5;
  text-decoration: none;
  font-family: SFRoundedRegular;
  scroll-behavior: smooth;
  scrollbar-width: thin;
  user-select: none;

  --os-handle-bg: #ADA9A1;
  --os-handle-bg-hover: #78756F;
  --os-handle-bg-active: #78756F;
}

html,
body,
#__nuxt {
  background-color: transparent;
}

.os-scrollbar-horizontal {
  display: none;
}
</style>