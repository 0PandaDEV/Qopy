<template>
  <div class="bg" @keydown.down.prevent="selectNext" @keydown.up.prevent="selectPrevious"
    @keydown.enter.prevent="pasteSelectedItem" tabindex="0">
    <input v-model="searchQuery" @input="searchHistory" autocorrect="off" autocapitalize="off" spellcheck="false"
      class="search" type="text" placeholder="Type to filter entries...">
    <div class="bottom-bar">
      <div class="left">
        <img src="/Logo.svg" alt="">
        <p>Clipboard Manager</p>
      </div>
      <div class="right">
        <div class="paste" @click="pasteSelectedItem">
          <p>Paste</p>
          <img src="/enter.svg" alt="">
        </div>
        <div class="divider"></div>
        <div class="actions">
          <p>Actions</p>
          <div>
            <img v-if="os == 'windows' || os == 'linux'" src="/ctrl.svg" alt="">
            <img v-if="os == 'macos'" src="/cmd.svg" alt="">
            <img src="/k.svg" alt="">
          </div>
        </div>
      </div>
    </div>
    <OverlayScrollbarsComponent class="results" ref="resultsContainer"
      :options="{ scrollbars: { autoHide: 'scroll' } }">
      <div v-for="(item, index) in filteredHistory" :key="item.id"
        :class="['result clothoid-corner', { 'selected': index === selectedIndex }]" @click="selectItem(index)"
        :ref="el => { if (index === selectedIndex) selectedElement = el }">
        <FileIcon />
        {{ truncateContent(item.content) }}
      </div>
    </OverlayScrollbarsComponent>
    <OverlayScrollbarsComponent class="content">
      {{ filteredHistory[selectedIndex]?.content || '' }}
    </OverlayScrollbarsComponent>
    <Noise />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { writeText, paste } from '@tauri-apps/plugin-clipboard-manager';
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import 'overlayscrollbars/overlayscrollbars.css';
import { app, window } from '@tauri-apps/api';
import { platform } from '@tauri-apps/plugin-os';
import { invoke } from '@tauri-apps/api/core';
import { enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { listen } from '@tauri-apps/api/event';

const db = ref(null);
const history = ref([]);
const searchQuery = ref('');
const selectedIndex = ref(0);
const resultsContainer = ref(null);
const selectedElement = ref(null);
const os = platform();

const filteredHistory = computed(() => {
  if (!searchQuery.value) return history.value;
  return history.value
    .filter(item => item.content.toLowerCase().includes(searchQuery.value.toLowerCase()))
    .sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp));
});

const searchHistory = () => {
  selectedIndex.value = 0;
};

const selectNext = () => {
  if (selectedIndex.value < filteredHistory.value.length - 1) {
    selectedIndex.value++;
    scrollToSelectedItem();
  }
};

const selectPrevious = () => {
  if (selectedIndex.value > 0) {
    selectedIndex.value--;
    scrollToSelectedItem();
  }
};

const scrollToSelectedItem = () => {
  nextTick(() => {
    if (selectedElement.value && resultsContainer.value) {
      const osInstance = resultsContainer.value.osInstance();
      const { viewport } = osInstance.elements();
      const element = selectedElement.value;

      const viewportRect = viewport.getBoundingClientRect();
      const elementRect = element.getBoundingClientRect();

      const isAbove = elementRect.top < viewportRect.top;
      const isBelow = elementRect.bottom > viewportRect.bottom - 48;

      if (isAbove || isBelow) {
        let scrollOffset;

        if (isAbove && selectedIndex.value === 0) {
          scrollOffset = elementRect.top - viewportRect.top - 14;
        } else if (isAbove) {
          scrollOffset = elementRect.top - viewportRect.top - 8;
        } else {
          scrollOffset = elementRect.bottom - viewportRect.bottom + 48;
        }

        viewport.scrollBy({
          top: scrollOffset,
          behavior: 'smooth'
        });
      }
    }
  });
};

const selectItem = (index) => {
  selectedIndex.value = index;
  scrollToSelectedItem();
};

const pasteSelectedItem = async () => {
  const selectedItem = filteredHistory.value[selectedIndex.value];
  if (selectedItem) {
    await writeText(selectedItem.content);
    await hideApp();
    await invoke("simulate_paste");
  }
};

const truncateContent = (content) => {
  const maxWidth = 284;
  const charWidth = 9;
  const maxChars = Math.floor(maxWidth / charWidth);
  return content.length > maxChars ? content.slice(0, maxChars - 3) + '...' : content;
};

onMounted(async () => {
  db.value = await Database.load('sqlite:data.db');
  await refreshHistory();

  if (!await isEnabled()) {
    await enable()
  }

  await listen('tauri://blur', hideApp);
});

const refreshHistory = async () => {
  history.value = await db.value.select('SELECT * FROM history ORDER BY timestamp DESC');
};

const hideApp = async () => {
  await app.hide();
  await window.getCurrent().hide();
};

const showApp = async () => {
  await refreshHistory();
  await app.show();
  await window.getCurrent().show();
  selectedIndex.value = 0;
};

watch(selectedIndex, scrollToSelectedItem);
</script>

<style lang="scss">
@import '~/assets/css/style.scss';
</style>
