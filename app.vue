<template>
  <div class="bg" @keydown.down.prevent="selectNext" @keydown.up.prevent="selectPrevious"
    @keydown.enter.prevent="pasteSelectedItem" @keydown.esc="hideApp" tabindex="0">
    <input ref="searchInput" v-model="searchQuery" @input="searchHistory" autocorrect="off" autocapitalize="off" spellcheck="false"
      class="search" type="text" placeholder="Type to filter entries...">
    <div class="bottom-bar">
      <div class="left">
        <img src="/Logo.svg" alt="">
        <p>Qopy</p>
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
      <template v-for="(group, groupIndex) in groupedHistory" :key="groupIndex">
        <div class="time-separator">{{ group.label }}</div>
        <div v-for="(item, index) in group.items" :key="item.id"
          :class="['result clothoid-corner', { 'selected': isSelected(groupIndex, index) }]"
          @click="selectItem(groupIndex, index)"
          :ref="el => { if (isSelected(groupIndex, index)) selectedElement = el }">
          <img v-if="isUrl(item.content)" :src="getFavicon(item.content)" alt="Favicon" class="favicon">
          <FileIcon v-else />
          <img v-if="item.type === 'image'" :src="item.content" alt="Image" class="preview-image">
          <span v-else>{{ truncateContent(item.content) }}</span>
        </div>
      </template>
    </OverlayScrollbarsComponent>
    <OverlayScrollbarsComponent class="content">
      <img v-if="selectedItem?.type === 'image'" :src="selectedItem.content" alt="Image" class="full-image">
      <img v-else-if="isYoutubeWatchUrl(selectedItem?.content)" :src="getYoutubeThumbnail(selectedItem.content)" alt="YouTube Thumbnail" class="full-image">
      <span v-else>{{ selectedItem?.content || '' }}</span>
    </OverlayScrollbarsComponent>
    <Noise />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import 'overlayscrollbars/overlayscrollbars.css';
import { app, window } from '@tauri-apps/api';
import { platform } from '@tauri-apps/plugin-os';
import { invoke } from '@tauri-apps/api/core';
import { enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { listen } from '@tauri-apps/api/event';
import { register, unregister, isRegistered } from '@tauri-apps/plugin-global-shortcut';

const db = ref(null);
const history = ref([]);
const searchQuery = ref('');
const selectedGroupIndex = ref(0);
const selectedItemIndex = ref(0);
const resultsContainer = ref(null);
const selectedElement = ref(null);
const searchInput = ref(null);
const os = platform();

const groupedHistory = computed(() => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  const getWeekNumber = (d) => {
    d = new Date(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()));
    d.setUTCDate(d.getUTCDate() + 4 - (d.getUTCDay() || 7));
    const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
    return Math.ceil((((d - yearStart) / 86400000) + 1) / 7);
  };

  const thisWeek = getWeekNumber(now);
  const thisYear = now.getFullYear();

  const groups = [
    { label: 'Today', items: [] },
    { label: 'Yesterday', items: [] },
    { label: 'This Week', items: [] },
    { label: 'Last Week', items: [] },
    { label: 'This Year', items: [] },
    { label: 'Last Year', items: [] },
  ];

  const filteredItems = searchQuery.value
    ? history.value.filter(item => item.content.toLowerCase().includes(searchQuery.value.toLowerCase()))
    : history.value;

  filteredItems.forEach(item => {
    const itemDate = new Date(item.timestamp);
    const itemWeek = getWeekNumber(itemDate);
    const itemYear = itemDate.getFullYear();

    if (itemDate.toDateString() === today.toDateString()) {
      groups[0].items.push(item);
    } else if (itemDate.toDateString() === new Date(today.getTime() - 86400000).toDateString()) {
      groups[1].items.push(item);
    } else if (itemYear === thisYear && itemWeek === thisWeek) {
      groups[2].items.push(item);
    } else if (itemYear === thisYear && itemWeek === thisWeek - 1) {
      groups[3].items.push(item);
    } else if (itemYear === thisYear) {
      groups[4].items.push(item);
    } else {
      groups[5].items.push(item);
    }
  });

  return groups.filter(group => group.items.length > 0);
});

const selectedItem = computed(() => {
  const group = groupedHistory.value[selectedGroupIndex.value];
  return group ? group.items[selectedItemIndex.value] : null;
});

const isSelected = (groupIndex, itemIndex) => {
  return selectedGroupIndex.value === groupIndex && selectedItemIndex.value === itemIndex;
};

const searchHistory = () => {
  selectedGroupIndex.value = 0;
  selectedItemIndex.value = 0;
};

const selectNext = () => {
  const currentGroup = groupedHistory.value[selectedGroupIndex.value];
  if (selectedItemIndex.value < currentGroup.items.length - 1) {
    selectedItemIndex.value++;
  } else if (selectedGroupIndex.value < groupedHistory.value.length - 1) {
    selectedGroupIndex.value++;
    selectedItemIndex.value = 0;
  }
  scrollToSelectedItem();
};

const selectPrevious = () => {
  if (selectedItemIndex.value > 0) {
    selectedItemIndex.value--;
  } else if (selectedGroupIndex.value > 0) {
    selectedGroupIndex.value--;
    selectedItemIndex.value = groupedHistory.value[selectedGroupIndex.value].items.length - 1;
  }
  scrollToSelectedItem();
};

const selectItem = (groupIndex, itemIndex) => {
  selectedGroupIndex.value = groupIndex;
  selectedItemIndex.value = itemIndex;
  scrollToSelectedItem();
};

const pasteSelectedItem = async () => {
  if (selectedItem.value) {
    await writeText(selectedItem.value.content);
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

const isUrl = (str) => {
  const urlPattern = /^(https?:\/\/)?(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)/;
  return urlPattern.test(str);
};

const isYoutubeWatchUrl = (url) => {
  return /^(https?:\/\/)?(www\.)?(youtube\.com|youtu\.be)\/watch\?v=[\w-]+/.test(url);
};

const getYoutubeThumbnail = (url) => {
  const videoId = url.match(/[?&]v=([^&]+)/)[1];
  return `https://img.youtube.com/vi/${videoId}/0.jpg`;
};

const getFavicon = (url) => {
  const domain = url.replace(/^(https?:\/\/)?(www\.)?/, '').split('/')[0];
  return `https://www.google.com/s2/favicons?domain=${domain}&sz=32`;
};

const refreshHistory = async () => {
  const rawHistory = await db.value.select('SELECT * FROM history ORDER BY timestamp DESC');
  history.value = rawHistory.map(item => {
    if (item.type === 'image' && !item.content.startsWith('data:image')) {
      return { ...item, content: `data:image/png;base64,${item.content}` };
    }
    return item;
  });
};

onMounted(async () => {
  db.value = await Database.load('sqlite:data.db');
  await refreshHistory();

  if (!await isEnabled()) {
    await enable()
  }

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

  await listen('tauri://blur', hideApp);
  await listen('tauri://focus', focusSearchInput);
  focusSearchInput();
});

const hideApp = async () => {
  await app.hide();
  await window.getCurrent().hide();
};

const showApp = async () => {
  await refreshHistory();
  await app.show();
  await window.getCurrent().show();
  selectedGroupIndex.value = 0;
  selectedItemIndex.value = 0;
  focusSearchInput();
};

const focusSearchInput = () => {
  nextTick(() => {
    searchInput.value?.focus();
  });
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
      const isBelow = elementRect.bottom > viewportRect.bottom - 8;

      if (isAbove || isBelow) {
        let scrollOffset;

        if (isAbove && selectedItemIndex.value === 0 && selectedGroupIndex.value === 0) {
          scrollOffset = elementRect.top - viewportRect.top - 36;
        } else if (isAbove) {
          scrollOffset = elementRect.top - viewportRect.top - 8;
        } else {
          scrollOffset = elementRect.bottom - viewportRect.bottom + 9;
        }

        viewport.scrollBy({
          top: scrollOffset,
          behavior: 'smooth'
        });
      }
    }
  });
};

watch([selectedGroupIndex, selectedItemIndex], scrollToSelectedItem);

</script>

<style lang="scss">
@import '~/assets/css/style.scss';
</style>
