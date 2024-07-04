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
    <div class="content">
      {{ filteredHistory[selectedIndex]?.content || '' }}
    </div>
    <Noise />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import 'overlayscrollbars/overlayscrollbars.css';
import { app, window } from '@tauri-apps/api';
import { platform } from '@tauri-apps/plugin-os';
import { invoke } from '@tauri-apps/api/core';
import { enable, isEnabled } from "@tauri-apps/plugin-autostart";

const history = ref([]);
const searchQuery = ref('');
const selectedIndex = ref(0);
const isVisible = ref(false);
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
    isVisible.value = false;
    await app.hide();
    await window.getCurrent().hide();
    await window.getCurrent().setFocus();
    await invoke('simulate_paste');
  }
};

const truncateContent = (content) => {
  const maxWidth = 284;
  const charWidth = 9;
  const maxChars = Math.floor(maxWidth / charWidth);
  return content.length > maxChars ? content.slice(0, maxChars - 3) + '...' : content;
};

onMounted(async () => {
  const db = await Database.load('sqlite:data.db');
  history.value = await db.select('SELECT * FROM history ORDER BY timestamp DESC');

  if (await isRegistered("MetaLeft+V")) {
    await unregister("MetaLeft+V")
  }

  await register('MetaLeft+V', (event) => {
    if (event.state === "MetaLeft+V") {
      if (isVisible.value == true) {
        app.hide()
        isVisible.value = false;
      } else {
        app.show()
        isVisible.value = true;
        selectedIndex.value = 0;
      }
    }
  });

  if (!await isEnabled()) {
    await enable()
  }
});

watch(selectedIndex, scrollToSelectedItem);
</script>

<style lang="scss">
@import '~/assets/css/style.scss';
</style>
