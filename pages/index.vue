<template>
  <div
    class="bg"
    @keydown.down.prevent="selectNext"
    @keydown.up.prevent="selectPrevious"
    @keydown.enter.prevent="pasteSelectedItem"
    @keydown.esc="hideApp"
    tabindex="0">
    <input
      ref="searchInput"
      v-model="searchQuery"
      @input="searchHistory"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      class="search"
      type="text"
      placeholder="Type to filter entries..." />
    <div class="bottom-bar">
      <div class="left">
        <img class="logo" width="18px" src="../public/logo.png" alt="" />
        <p>Qopy</p>
      </div>
      <div class="right">
        <div class="paste" @click="pasteSelectedItem">
          <p>Paste</p>
          <img src="../public/enter.svg" alt="" />
        </div>
        <div class="divider"></div>
        <div class="actions">
          <p>Actions</p>
          <div>
            <img
              v-if="os === 'windows' || os === 'linux'"
              src="../public/ctrl.svg"
              alt="" />
            <img v-if="os === 'macos'" src="../public/cmd.svg" alt="" />
            <img src="../public/k.svg" alt="" />
          </div>
        </div>
      </div>
    </div>
    <OverlayScrollbarsComponent
      class="results"
      ref="resultsContainer"
      :options="{ scrollbars: { autoHide: 'scroll' } }">
      <template v-for="(group, groupIndex) in groupedHistory" :key="groupIndex">
        <div class="time-separator">{{ group.label }}</div>
        <div
          v-for="(item, index) in group.items"
          :key="item.id"
          :class="[
            'result clothoid-corner',
            { selected: isSelected(groupIndex, index) },
          ]"
          @click="selectItem(groupIndex, index)"
          :ref="
            (el) => {
              if (isSelected(groupIndex, index))
                selectedElement = el as HTMLElement;
            }
          ">
          <template v-if="item.content_type === 'image'">
            <img
              v-if="imageUrls[item.id]"
              :src="imageUrls[item.id]"
              alt="Image"
              class="image"
              @error="onImageError" />
            <img
              v-else
              src="../public/icons/Image.svg"
              class="icon" />
          </template>
          <img
            v-else-if="hasFavicon(item.favicon ?? '')"
            :src="getFaviconFromDb(item.favicon ?? '')"
            alt="Favicon"
            class="favicon" />
          <img
            src="../public/icons/File.svg"
            class="icon"
            v-else-if="item.content_type === ContentType.File" />
          <img
            src="../public/icons/Text.svg"
            class="icon"
            v-else-if="item.content_type === ContentType.Text" />
          <img
            src="../public/icons/Code.svg"
            class="icon"
            v-else-if="item.content_type === ContentType.Code" />
          <span v-if="item.content_type === ContentType.Image">
            Image ({{ imageDimensions[item.id] || 'Loading...' }})
          </span>
          <span v-else>{{ truncateContent(item.content) }}</span>
        </div>
      </template>
    </OverlayScrollbarsComponent>
    <div class="content" v-if="selectedItem?.content_type === 'image'">
      <img :src="getComputedImageUrl(selectedItem)" alt="Image" class="image" />
    </div>
    <OverlayScrollbarsComponent v-else class="content">
      <img
        v-if="selectedItem?.content && isYoutubeWatchUrl(selectedItem.content)"
        :src="getYoutubeThumbnail(selectedItem.content)"
        alt="YouTube Thumbnail"
        class="full-image" />
      <span v-else>{{ selectedItem?.content || "" }}</span>
    </OverlayScrollbarsComponent>
    <Noise />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, nextTick, shallowRef } from "vue";
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import "overlayscrollbars/overlayscrollbars.css";
import { app, window } from "@tauri-apps/api";
import { platform } from "@tauri-apps/plugin-os";
import { enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { listen } from "@tauri-apps/api/event";
import { useNuxtApp } from "#app";
import { HistoryItem, ContentType } from "~/types/types";

interface GroupedHistory {
  label: string;
  items: HistoryItem[];
}

const { $history, $settings } = useNuxtApp();
const CHUNK_SIZE = 50;
const SCROLL_THRESHOLD = 100;
const IMAGE_LOAD_DEBOUNCE = 300;

const history = shallowRef<HistoryItem[]>([]);
let offset = 0;
let isLoading = false;

const resultsContainer = shallowRef<InstanceType<typeof OverlayScrollbarsComponent> | null>(null);
const searchQuery = ref("");
const selectedGroupIndex = ref(0);
const selectedItemIndex = ref(0);
const selectedElement = shallowRef<HTMLElement | null>(null);
const searchInput = ref<HTMLInputElement | null>(null);
const os = ref<string>("");
const imageUrls = shallowRef<Record<string, string>>({});
const imageDimensions = shallowRef<Record<string, string>>({});
const lastUpdateTime = ref<number>(Date.now());
const imageLoadError = ref<boolean>(false);
const imageLoading = ref<boolean>(false);

const isSameDay = (date1: Date, date2: Date): boolean => {
  return date1.getFullYear() === date2.getFullYear() 
    && date1.getMonth() === date2.getMonth() 
    && date1.getDate() === date2.getDate();
};

const getWeekNumber = (date: Date): number => {
  const firstDayOfYear = new Date(date.getFullYear(), 0, 1);
  return Math.ceil(((date.getTime() - firstDayOfYear.getTime()) / 86400000 + firstDayOfYear.getDay() + 1) / 7);
};

const groupedHistory = computed<GroupedHistory[]>(() => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const thisWeek = getWeekNumber(now);
  const thisYear = now.getFullYear();

  const groups: Record<string, HistoryItem[]> = {
    Today: [],
    Yesterday: [],
    'This Week': [],
    'Last Week': [],
    'This Year': [],
    'Last Year': []
  };

  const filteredItems = searchQuery.value
    ? history.value.filter(item => 
        item.content.toLowerCase().includes(searchQuery.value.toLowerCase()))
    : history.value;

  const yesterday = new Date(today.getTime() - 86400000);

  filteredItems.forEach(item => {
    const itemDate = new Date(item.timestamp);
    const itemWeek = getWeekNumber(itemDate);
    const itemYear = itemDate.getFullYear();

    if (isSameDay(itemDate, today)) groups.Today.push(item);
    else if (isSameDay(itemDate, yesterday)) groups.Yesterday.push(item);
    else if (itemYear === thisYear && itemWeek === thisWeek) groups['This Week'].push(item);
    else if (itemYear === thisYear && itemWeek === thisWeek - 1) groups['Last Week'].push(item);
    else if (itemYear === thisYear) groups['This Year'].push(item);
    else groups['Last Year'].push(item);
  });

  return Object.entries(groups)
    .filter(([_, items]) => items.length > 0)
    .map(([label, items]) => ({ label, items }));
});

const selectedItem = computed<HistoryItem | null>(() => {
  const group = groupedHistory.value[selectedGroupIndex.value];
  return group?.items[selectedItemIndex.value] ?? null;
});

const loadHistoryChunk = async (): Promise<void> => {
  if (isLoading) return;
  isLoading = true;

  try {
    const results = await $history.loadHistoryChunk(offset, CHUNK_SIZE);
    if (!results.length) {
      isLoading = false;
      return;
    }

    const processedItems = await Promise.all(
      results.map(async item => {
        const historyItem = new HistoryItem(
          item.content_type as ContentType,
          item.content,
          item.favicon
        );
        Object.assign(historyItem, {
          id: item.id,
          timestamp: new Date(item.timestamp)
        });

        if (historyItem.content_type === ContentType.Image) {
          await Promise.all([
            getItemDimensions(historyItem),
            loadImageUrl(historyItem)
          ]);
        }
        return historyItem;
      })
    );

    history.value = [...history.value, ...processedItems];
    offset += CHUNK_SIZE;
  } catch (error) {
    console.error("Failed to load history:", error);
  } finally {
    isLoading = false;
  }
};

const handleScroll = (): void => {
  const viewport = resultsContainer.value?.osInstance()?.elements().viewport;
  if (!viewport) return;

  const { scrollTop, scrollHeight, clientHeight } = viewport;
  if (scrollHeight - scrollTop - clientHeight < SCROLL_THRESHOLD) {
    loadHistoryChunk();
  }
};

const scrollToSelectedItem = (forceScrollTop: boolean = false): void => {
  nextTick(() => {
    const osInstance = resultsContainer.value?.osInstance();
    const viewport = osInstance?.elements().viewport;
    if (!selectedElement.value || !viewport) return;

    if (!forceScrollTop) {
      const viewportRect = viewport.getBoundingClientRect();
      const elementRect = selectedElement.value.getBoundingClientRect();
      
      const isAbove = elementRect.top < viewportRect.top;
      const isBelow = elementRect.bottom > viewportRect.bottom - 8;

      if (isAbove || isBelow) {
        const scrollOffset = isAbove
          ? elementRect.top - viewportRect.top - 8
          : elementRect.bottom - viewportRect.bottom + 9;

        viewport.scrollBy({ top: scrollOffset, behavior: "smooth" });
      }
    }
  });
};

const isSelected = (groupIndex: number, itemIndex: number): boolean => {
  return (
    selectedGroupIndex.value === groupIndex &&
    selectedItemIndex.value === itemIndex
  );
};

const searchHistory = async (): Promise<void> => {
  const results = await $history.searchHistory(searchQuery.value);
  history.value = results.map(item => Object.assign(
    new HistoryItem(item.content_type as ContentType, item.content, item.favicon),
    { id: item.id, timestamp: new Date(item.timestamp) }
  ));
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
    selectedItemIndex.value =
      groupedHistory.value[selectedGroupIndex.value].items.length - 1;
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
  let contentType: string = selectedItem.value.content_type;
  if (contentType === "image") {
    try {
      content = await $history.getImagePath(content);
    } catch (error) {
      console.error("Error reading image file:", error);
      return;
    }
  }
  await hideApp();
  await $history.writeAndPaste({ content, contentType });
};

const truncateContent = (content: string): string => {
  const maxWidth = 284;
  const charWidth = 9;
  const maxChars = Math.floor(maxWidth / charWidth);
  return content.length > maxChars
    ? content.slice(0, maxChars - 3) + "..."
    : content;
};

const hasFavicon = (str: string): boolean => {
  return str.trim() !== "";
};

const isYoutubeWatchUrl = (url: string): boolean => {
  return (
    /^(https?:\/\/)?(www\.)?(youtube\.com|youtu\.be)\/watch\?v=[\w-]+/.test(
      url
    ) || /^(https?:\/\/)?(www\.)?youtu\.be\/[\w-]+/.test(url)
  );
};

const getYoutubeThumbnail = (url: string): string => {
  let videoId;
  if (url.includes("youtu.be")) {
    videoId = url.split("youtu.be/")[1];
  } else {
    videoId = url.match(/[?&]v=([^&]+)/)?.[1];
  }
  return videoId
    ? `https://img.youtube.com/vi/${videoId}/0.jpg`
    : "https://via.placeholder.com/150";
};

const getFaviconFromDb = (favicon: string): string => {
  return `data:image/png;base64,${favicon}`;
};

const getImageData = async (item: HistoryItem): Promise<{ url: string; dimensions: string }> => {
  try {
    const base64 = await $history.readImage({ filename: item.content });
    const dataUrl = `data:image/png;base64,${base64}`;
    const img = new Image();
    img.src = dataUrl;
    
    await new Promise<void>((resolve, reject) => {
      img.onload = () => resolve();
      img.onerror = reject;
    });

    return {
      url: dataUrl,
      dimensions: `${img.width}x${img.height}`
    };
  } catch (error) {
    console.error("Error processing image:", error);
    return { url: "", dimensions: "Error" };
  }
};

const processHistoryItem = async (item: any): Promise<HistoryItem> => {
  const historyItem = new HistoryItem(
    item.content_type as ContentType,
    item.content,
    item.favicon
  );
  
  Object.assign(historyItem, {
    id: item.id,
    timestamp: new Date(item.timestamp)
  });

  if (historyItem.content_type === ContentType.Image) {
    const { url, dimensions } = await getImageData(historyItem);
    imageUrls.value[historyItem.id] = url;
    imageDimensions.value[historyItem.id] = dimensions;
  }

  return historyItem;
};

const updateHistory = async (resetScroll: boolean = false): Promise<void> => {
  history.value = [];
  offset = 0;
  await loadHistoryChunk();
  
  if (resetScroll && resultsContainer.value?.osInstance()?.elements().viewport) {
    resultsContainer.value.osInstance()?.elements().viewport?.scrollTo({
      top: 0,
      behavior: "smooth"
    });
  }
};

const handleSelection = (groupIndex: number, itemIndex: number, shouldScroll: boolean = true): void => {
  selectedGroupIndex.value = groupIndex;
  selectedItemIndex.value = itemIndex;
  if (shouldScroll) scrollToSelectedItem();
};

const handleMediaContent = async (content: string, type: string): Promise<string> => {
  if (type === "image") {
    return await $history.getImagePath(content);
  }
  
  if (isYoutubeWatchUrl(content)) {
    const videoId = content.includes("youtu.be")
      ? content.split("youtu.be/")[1]
      : content.match(/[?&]v=([^&]+)/)?.[1];
    return videoId ? `https://img.youtube.com/vi/${videoId}/0.jpg` : "";
  }
  
  return content;
};

const setupEventListeners = async (): Promise<void> => {
  await listen("clipboard-content-updated", async () => {
    lastUpdateTime.value = Date.now();
    handleSelection(0, 0, false);
    await updateHistory(true);
  });

  await listen("tauri://focus", async () => {
    const currentTime = Date.now();
    if (currentTime - lastUpdateTime.value > 0) {
      const previousState = {
        groupIndex: selectedGroupIndex.value,
        itemIndex: selectedItemIndex.value,
        scroll: resultsContainer.value?.osInstance()?.elements().viewport?.scrollTop || 0
      };
      
      await updateHistory();
      lastUpdateTime.value = currentTime;
      handleSelection(previousState.groupIndex, previousState.itemIndex, false);
      
      nextTick(() => {
        const viewport = resultsContainer.value?.osInstance()?.elements().viewport;
        if (viewport) {
          viewport.scrollTo({
            top: previousState.scroll,
            behavior: "instant"
          });
        }
      });
    }
    focusSearchInput();
  });

  await listen("tauri://blur", () => {
    searchInput.value?.blur();
  });
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

const onImageError = (): void => {
  imageLoadError.value = true;
  imageLoading.value = false;
};

watch([selectedGroupIndex, selectedItemIndex], () => {
  scrollToSelectedItem();
});

watch(searchQuery, () => {
  searchHistory();
});

onMounted(async () => {
  try {
    os.value = await platform();
    await loadHistoryChunk();

    resultsContainer.value
      ?.osInstance()
      ?.elements()
      ?.viewport?.addEventListener("scroll", handleScroll);

    await setupEventListeners();

    if (!(await isEnabled())) {
      await enable();
    }
  } catch (error) {
    console.error("Error during onMounted:", error);
  }
});

watch([selectedGroupIndex, selectedItemIndex], () =>
  scrollToSelectedItem(false)
);

const getItemDimensions = async (item: HistoryItem) => {
  if (!imageDimensions.value[item.id]) {
    try {
      const base64 = await $history.readImage({ filename: item.content });
      const img = new Image();
      img.src = `data:image/png;base64,${base64}`;
      await new Promise<void>((resolve, reject) => {
        img.onload = () => resolve();
        img.onerror = () => reject();
      });
      imageDimensions.value[item.id] = `${img.width}x${img.height}`;
    } catch (error) {
      console.error("Error loading image dimensions:", error);
      imageDimensions.value[item.id] = "Error";
    }
  }
  return imageDimensions.value[item.id] || 'Loading...';
};

const loadImageUrl = async (item: HistoryItem) => {
  if (!imageUrls.value[item.id]) {
    try {
      const base64 = await $history.readImage({ filename: item.content });
      imageUrls.value[item.id] = `data:image/png;base64,${base64}`;
    } catch (error) {
      console.error("Error loading image:", error);
    }
  }
};

const getComputedImageUrl = (item: HistoryItem | null): string => {
  if (!item) return '';
  return imageUrls.value[item.id] || '';
};
</script>

<style scoped lang="scss">
@use "~/assets/css/index.scss";
</style>
