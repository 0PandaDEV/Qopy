<template>
  <div class="bg" @keydown.down.prevent="selectNext" @keydown.up.prevent="selectPrevious"
    @keydown.enter.prevent="pasteSelectedItem" @keydown.esc="hideApp" tabindex="0">
    <input ref="searchInput" v-model="searchQuery" @input="searchHistory" autocorrect="off" autocapitalize="off"
      spellcheck="false" class="search" type="text" placeholder="Type to filter entries...">
    <div class="bottom-bar">
      <div class="left">
        <img class="logo" width="18px" src="/logo.png" alt="">
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
            <img v-if="os === 'windows' || os === 'linux'" src="/ctrl.svg" alt="">
            <img v-if="os === 'macos'" src="/cmd.svg" alt="">
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
          :ref="el => { if (isSelected(groupIndex, index)) selectedElement = el as HTMLElement }">
          <img v-if="item.content_type === 'image'" :src="getComputedImageUrl(item)" alt="Image" class="favicon-image">
          <img v-else-if="hasFavicon(item.favicon ?? '')" :src="getFaviconFromDb(item.favicon ?? '')" alt="Favicon" class="favicon">
          <FileIcon class="file" v-else />
          <span v-if="item.content_type === 'image'">Image ({{ item.dimensions || 'Loading...' }})</span>
          <span v-else>{{ truncateContent(item.content) }}</span>
        </div>
      </template>
    </OverlayScrollbarsComponent>
    <div class="content" v-if="selectedItem?.content_type === 'image'">
      <img :src="getComputedImageUrl(selectedItem)" alt="Image" class="image">
    </div>
    <OverlayScrollbarsComponent v-else class="content">
      <img v-if="selectedItem?.content && isYoutubeWatchUrl(selectedItem.content)" :src="getYoutubeThumbnail(selectedItem.content)"
        alt="YouTube Thumbnail" class="full-image">
      <span v-else>{{ selectedItem?.content || '' }}</span>
    </OverlayScrollbarsComponent>
    <Noise />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick, shallowRef } from 'vue';
import Database from '@tauri-apps/plugin-sql';
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import 'overlayscrollbars/overlayscrollbars.css';
import { app, window } from '@tauri-apps/api';
import { platform } from '@tauri-apps/plugin-os';
import { invoke } from '@tauri-apps/api/core';
import { enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { listen } from '@tauri-apps/api/event';
import { readFile } from '@tauri-apps/plugin-fs';

interface HistoryItem {
  id: number;
  content: string;
  content_type: string;
  timestamp: string;
  favicon?: string;
  dimensions?: string;
}

interface GroupedHistory {
  label: string;
  items: HistoryItem[];
}

const db: Ref<Database | null> = ref(null);
const history: Ref<HistoryItem[]> = ref([]);
const chunkSize: number = 50;
let offset: number = 0;
let isLoading: boolean = false;
const resultsContainer: Ref<InstanceType<typeof OverlayScrollbarsComponent> | null> = ref(null);
const searchQuery: Ref<string> = ref('');
const selectedGroupIndex: Ref<number> = ref(0);
const selectedItemIndex: Ref<number> = ref(0);
const selectedElement: Ref<HTMLElement | null> = ref(null);
const searchInput: Ref<HTMLInputElement | null> = ref(null);
const os: Ref<string> = ref('');

const groupedHistory: ComputedRef<GroupedHistory[]> = computed(() => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());

  const getWeekNumber = (d: Date): number => {
    d = new Date(Date.UTC(d.getFullYear(), d.getMonth(), d.getDate()));
    d.setUTCDate(d.getUTCDate() + 4 - (d.getUTCDay() || 7));
    const yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
    return Math.ceil(((Number(d) - Number(yearStart)) / 86400000 + 1) / 7);
  };

  const thisWeek = getWeekNumber(now);
  const thisYear = now.getFullYear();

  const groups: GroupedHistory[] = [
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

const selectedItem: ComputedRef<HistoryItem | null> = computed(() => {
  const group = groupedHistory.value[selectedGroupIndex.value];
  return group ? group.items[selectedItemIndex.value] : null;
});

const isSelected = (groupIndex: number, itemIndex: number): boolean => {
  return selectedGroupIndex.value === groupIndex && selectedItemIndex.value === itemIndex;
};

const searchHistory = async (): Promise<void> => {
  if (!db.value) return;

  history.value = [];
  offset = 0;

  const query = `%${searchQuery.value}%`;
  const results = await db.value.select<HistoryItem[]>(
    'SELECT * FROM history WHERE content LIKE ? ORDER BY timestamp DESC LIMIT ?',
    [query, chunkSize]
  );

  history.value = await Promise.all(results.map(async item => {
    if (item.content_type === 'image') {
      const dimensions = await getImageDimensions(item.content);
      return { ...item, dimensions };
    }
    return item;
  }));
};

const selectNext = (): void => {
  const currentGroup = groupedHistory.value[selectedGroupIndex.value];
  if (selectedItemIndex.value < currentGroup.items.length - 1) {
    selectedItemIndex.value++;
  } else if (selectedGroupIndex.value < groupedHistory.value.length - 1) {
    selectedGroupIndex.value++;
    selectedItemIndex.value = 0;
  }
  scrollToSelectedItem();
};

const selectPrevious = (): void => {
  if (selectedItemIndex.value > 0) {
    selectedItemIndex.value--;
  } else if (selectedGroupIndex.value > 0) {
    selectedGroupIndex.value--;
    selectedItemIndex.value = groupedHistory.value[selectedGroupIndex.value].items.length - 1;
  }
  scrollToSelectedItem();
};

const selectItem = (groupIndex: number, itemIndex: number): void => {
  selectedGroupIndex.value = groupIndex;
  selectedItemIndex.value = itemIndex;
  scrollToSelectedItem();
};

const pasteSelectedItem = async (): Promise<void> => {
  if (!selectedItem.value) return;
  
  let content = selectedItem.value.content;
  let contentType: String = selectedItem.value.content_type;
  if (contentType === 'image') {
    try {
      content = readFile(content).toString();
    } catch (error) {
      console.error('Error reading image file:', error);
      return;
    }
  }
  await hideApp();
  await invoke("write_and_paste", { 
    content, 
    contentType 
  });
};

const truncateContent = (content: string): string => {
  const maxWidth = 284;
  const charWidth = 9;
  const maxChars = Math.floor(maxWidth / charWidth);
  return content.length > maxChars ? content.slice(0, maxChars - 3) + '...' : content;
};

const hasFavicon = (str: string): boolean => {
  return str.trim() !== '';
};

const isYoutubeWatchUrl = (url: string): boolean => {
  return /^(https?:\/\/)?(www\.)?(youtube\.com|youtu\.be)\/watch\?v=[\w-]+/.test(url) || /^(https?:\/\/)?(www\.)?youtu\.be\/[\w-]+/.test(url);
};

const getYoutubeThumbnail = (url: string): string => {
  let videoId;
  if (url.includes('youtu.be')) {
    videoId = url.split('youtu.be/')[1];
  } else {
    videoId = url.match(/[?&]v=([^&]+)/)?.[1];
  }
  return `https://img.youtube.com/vi/${videoId}/0.jpg`;
};

const getFaviconFromDb = (favicon: string): string => {
  return `data:image/png;base64,${favicon}`;
};

const getImageDimensions = (path: string): Promise<string> => {
  return new Promise(async (resolve) => {
    const img = new Image();
    img.onload = () => resolve(`${img.width}x${img.height}`);
    img.onerror = () => resolve('0x0');
    if (path.includes('AppData\\Roaming\\net.pandadev.qopy\\images\\')) {
      const filename = path.split('\\').pop();
      try {
        const imageData = await invoke<Uint8Array>("read_image", { filename: filename });
        const blob = new Blob([imageData], { type: 'image/png' });
        img.src = URL.createObjectURL(blob);
      } catch (error) {
        console.error('Error reading image file:', error);
        resolve('0x0');
      }
    } else {
      img.src = `data:image/png;base64,${path}`;
    }
  });
};

const imageUrls: Ref<Record<number, string>> = shallowRef({});

const getComputedImageUrl = (item: HistoryItem): string => {
  if (!imageUrls.value[item.id]) {
    imageUrls.value[item.id] = '';
    getImageUrl(item.content).then(url => {
      imageUrls.value = { ...imageUrls.value, [item.id]: url };
    });
  }
  return imageUrls.value[item.id] || '';
};

const getImageUrl = async (path: string): Promise<string> => {
  if (path.includes('AppData\\Roaming\\net.pandadev.qopy\\images\\')) {
    const filename = path.split('\\').pop();
    try {
      const imageData = await invoke<Uint8Array>("read_image", { filename: filename });
      const blob = new Blob([imageData], { type: 'image/png' });
      return URL.createObjectURL(blob);
    } catch (error) {
      console.error('Error reading image file:', error);
      return '';
    }
  } else {
    return `data:image/png;base64,${path}`;
  }
};

const loadHistoryChunk = async (): Promise<void> => {
  if (!db.value || isLoading) return;

  isLoading = true;
  let results: HistoryItem[];

  if (searchQuery.value) {
    const query = `%${searchQuery.value}%`;
    results = await db.value.select(
      'SELECT * FROM history WHERE content LIKE ? ORDER BY timestamp DESC LIMIT ? OFFSET ?',
      [query, chunkSize, offset]
    );
  } else {
    results = await db.value.select(
      'SELECT * FROM history ORDER BY timestamp DESC LIMIT ? OFFSET ?',
      [chunkSize, offset]
    );
  }

  if (results.length === 0) {
    isLoading = false;
    return;
  }

  const processedChunk = await Promise.all(results.map(async item => {
    if (item.content_type === 'image') {
      const dimensions = await getImageDimensions(item.content);
      return { ...item, dimensions };
    }
    return item;
  }));

  history.value = [...history.value, ...processedChunk];
  offset += chunkSize;
  isLoading = false;
};

const handleScroll = (): void => {
  if (!resultsContainer.value) return;
  
  const { viewport } = resultsContainer.value?.osInstance().elements() ?? {};
  const { scrollTop = 0, scrollHeight = 0, clientHeight = 0 } = viewport ?? {};
  
  if (scrollHeight - scrollTop - clientHeight < 100) {
    loadHistoryChunk();
  }
};

const hideApp = async (): Promise<void> => {
  await app.hide();
  await window.getCurrentWindow().hide();
};

const focusSearchInput = (): void => {
  nextTick(() => {
    searchInput.value?.focus();
  });
};

const scrollToSelectedItem = (): void => {
  nextTick(() => {
    if (selectedElement.value && resultsContainer.value) {
      const osInstance = resultsContainer.value.osInstance();
      const viewport = osInstance?.elements().viewport;
      if (!viewport) return;

      const viewportRect = viewport.getBoundingClientRect();
      const elementRect = selectedElement.value.getBoundingClientRect();

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

watch(searchQuery, () => {
  searchHistory();
});

onMounted(async () => {
  db.value = await Database.load('sqlite:data.db');
  await loadHistoryChunk();

  if (resultsContainer.value) {
    resultsContainer.value.osInstance().elements().viewport.addEventListener('scroll', handleScroll);
  }

  await listen('tauri://focus', async () => {
    history.value = [];
    offset = 0;
    await loadHistoryChunk();
    focusSearchInput();
  });

  await listen('tauri://blur', () => {
    if (searchInput.value) {
      searchInput.value.blur();
    }
  });

  if (!await isEnabled()) {
    await enable()
  }

  os.value = await platform();
});

</script>

<style lang="scss">
@import '~/assets/css/style.scss';
</style>