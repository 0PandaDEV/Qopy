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
    await navigateTo('/keybind')
    await app.show();
    await window.getCurrentWindow().show();
    manageFocus();
  })

  await listen('main_route', async () => {
    await navigateTo('/')
  })
})

function manageFocus() {
  if (process.platform === 'win32') {
    const { SetForegroundWindow, AttachThreadInput, GetForegroundWindow, GetWindowThreadProcessId } = require('windows-api');
    const foregroundWindow = GetForegroundWindow();
    const currentThreadId = GetWindowThreadProcessId(foregroundWindow, null);
    const targetThreadId = GetWindowThreadProcessId(window.hwnd(), null);

    AttachThreadInput(currentThreadId, targetThreadId, 1);
    SetForegroundWindow(window.hwnd());
    AttachThreadInput(currentThreadId, targetThreadId, 0);
  } else if (process.platform === 'darwin') {
    const { NSWindow } = require('cocoa');
    const nsWindow = window.ns_window();
    nsWindow.makeKeyAndOrderFront(true);
  } else if (process.platform === 'linux') {
    const { XOpenDisplay, XDefaultRootWindow, XSetInputFocus, XCloseDisplay, RevertToParent } = require('xlib');
    const display = XOpenDisplay(null);
    const rootWindow = XDefaultRootWindow(display);
    XSetInputFocus(display, rootWindow, RevertToParent, 0);
    XCloseDisplay(display);
  }
}
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
