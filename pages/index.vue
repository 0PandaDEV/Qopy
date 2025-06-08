<template>
  <main>
    <TopBar
      ref="topBar"
      @search="searchHistory"
      @searchStarted="searchStarted" />
    <div class="container">
      <OverlayScrollbarsComponent
        class="results"
        ref="resultsContainer"
        :options="{ scrollbars: { autoHide: 'scroll' } }">
        <div
          v-for="(group, groupIndex) in groupedHistory"
          :key="groupIndex"
          class="group">
          <div class="time-separator">{{ group.label }}</div>
          <div class="results-group">
            <Result
              v-for="(item, index) in group.items"
              :key="item.id"
              :item="item"
              :selected="isSelected(groupIndex, index)"
              :image-url="imageUrls[item.id]"
              :dimensions="imageDimensions[item.id]"
              @select="selectItem(groupIndex, index)"
              @image-error="onImageError"
              @setRef="(el: HTMLElement | null) => (selectedElement = el)" />
          </div>
        </div>
      </OverlayScrollbarsComponent>
      <div class="right">
        <div
          class="content"
          v-if="selectedItem?.content_type === ContentType.Image">
          <img :src="imageUrls[selectedItem.id]" alt="Image" class="image" />
        </div>
        <div
          v-else-if="selectedItem && isYoutubeWatchUrl(selectedItem.content)"
          class="content">
          <img
            class="image"
            :src="getYoutubeThumbnail(selectedItem.content)"
            alt="YouTube Thumbnail" />
        </div>
        <div
          class="content"
          v-else-if="
            selectedItem?.content_type === ContentType.Link && pageOgImage
          ">
          <img :src="pageOgImage" alt="Image" class="image" />
        </div>
        <OverlayScrollbarsComponent v-else class="content">
          <span class="content-text">{{ selectedItem?.content || "" }}</span>
        </OverlayScrollbarsComponent>
        <OverlayScrollbarsComponent
          class="information"
          :options="{ scrollbars: { autoHide: 'scroll' } }">
          <div class="title">Information</div>
          <div class="info-content" v-if="selectedItem && getInfo">
            <div class="info-row" v-for="(row, index) in infoRows" :key="index">
              <p class="label">{{ row.label }}</p>
              <span
                :class="{ 'url-truncate': row.isUrl }"
                :data-text="row.value">
                <img v-if="row.icon" :src="row.icon" :alt="String(row.value)" />
                {{ row.value }}
              </span>
            </div>
          </div>
        </OverlayScrollbarsComponent>
      </div>
    </div>
    <BottomBar
      :primary-action="{
        text: 'Paste',
        icon: IconsEnter,
        onClick: pasteSelectedItem,
      }"
      :secondary-action="{
        text: 'Actions',
        icon: IconsKey,
        input: 'K',
        showModifier: true,
        onClick: toggleActionsMenu,
      }" />
    <ActionsMenu
      :selected-item="selectedItem"
      :is-visible="isActionsMenuVisible"
      @close="closeActionsMenu"
      @toggle="toggleActionsMenu" />
  </main>
</template>

<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  watch,
  nextTick,
  shallowRef,
} from "vue";
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import "overlayscrollbars/overlayscrollbars.css";
import { listen } from "@tauri-apps/api/event";
import { useNuxtApp } from "#app";
import { invoke } from "@tauri-apps/api/core";
import { HistoryItem, ContentType } from "~/types/types";
import type {
  InfoText,
  InfoImage,
  InfoFile,
  InfoLink,
  InfoColor,
  InfoCode,
} from "~/types/types";
import IconsEnter from "~/components/Keys/Enter.vue";
import IconsKey from "~/components/Keys/Key.vue";
import ActionsMenu from "~/components/ActionsMenu.vue";
import { useAppControl } from "~/composables/useAppControl";

interface GroupedHistory {
  label: string;
  items: HistoryItem[];
}

const { $history, $keyboard, $selectedResult } = useNuxtApp();
const {
  selectedGroupIndex,
  selectedItemIndex,
  selectedElement,
  useSelectedResult,
} = $selectedResult;

const listeners: Array<() => void> = [];

const CHUNK_SIZE = 50;
const SCROLL_THRESHOLD = 100;
const SCROLL_PADDING = 8;
const TOP_SCROLL_PADDING = 37;

const history = shallowRef<HistoryItem[]>([]);
let offset = 0;
let isLoading = false;

const resultsContainer = shallowRef<InstanceType<
  typeof OverlayScrollbarsComponent
> | null>(null);
const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const imageUrls = shallowRef<Record<string, string>>({});
const imageDimensions = shallowRef<Record<string, string>>({});
const imageSizes = shallowRef<Record<string, string>>({});
const lastUpdateTime = ref<number>(Date.now());
const imageLoadError = ref<boolean>(false);
const imageLoading = ref<boolean>(false);
const pageTitle = ref<string>("");
const pageOgImage = ref<string>("");
const isActionsMenuVisible = ref<boolean>(false);

const topBar = ref<{ searchInput: HTMLInputElement | null } | null>(null);

const toggleActionsMenu = () => {
  isActionsMenuVisible.value = !isActionsMenuVisible.value;
  nextTick(() => {
    if (isActionsMenuVisible.value) {
      document.getElementById("actions-menu")?.focus();
    } else {
      focusSearchInput();
    }
  });
};

const closeActionsMenu = () => {
  isActionsMenuVisible.value = false;
  focusSearchInput();
};

const isSameDay = (date1: Date, date2: Date): boolean => {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
};

const getWeekNumber = (date: Date): number => {
  const firstDayOfYear = new Date(date.getFullYear(), 0, 1);
  return Math.ceil(
    ((date.getTime() - firstDayOfYear.getTime()) / 86400000 +
      firstDayOfYear.getDay() +
      1) /
      7
  );
};

const groupedHistory = computed<GroupedHistory[]>(() => {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const thisWeek = getWeekNumber(now);
  const thisYear = now.getFullYear();

  const groups: Record<string, HistoryItem[]> = {
    Today: [],
    Yesterday: [],
    "This Week": [],
    "Last Week": [],
    "This Year": [],
    "Last Year": [],
  };

  const filteredItems = searchQuery.value
    ? history.value.filter((item) =>
        item.content.toLowerCase().includes(searchQuery.value.toLowerCase())
      )
    : history.value;

  const yesterday = new Date(today.getTime() - 86400000);

  filteredItems.forEach((item) => {
    const itemDate = new Date(item.timestamp);
    const itemWeek = getWeekNumber(itemDate);
    const itemYear = itemDate.getFullYear();

    if (isSameDay(itemDate, today)) groups.Today.push(item);
    else if (isSameDay(itemDate, yesterday)) groups.Yesterday.push(item);
    else if (itemYear === thisYear && itemWeek === thisWeek)
      groups["This Week"].push(item);
    else if (itemYear === thisYear && itemWeek === thisWeek - 1)
      groups["Last Week"].push(item);
    else if (itemYear === thisYear) groups["This Year"].push(item);
    else groups["Last Year"].push(item);
  });

  return Object.entries(groups)
    .filter(([_, items]) => items.length > 0)
    .map(([label, items]) => ({ label, items }));
});

const { selectedItem, isSelected, selectNext, selectPrevious, selectItem } =
  useSelectedResult(groupedHistory);

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
      results.map(async (item) => {
        const historyItem = new HistoryItem(
          item.source,
          item.content_type,
          item.content,
          item.favicon,
          item.source_icon,
          item.language
        );
        Object.assign(historyItem, {
          id: item.id,
          timestamp: new Date(item.timestamp),
        });

        if (historyItem.content_type === ContentType.Image) {
          try {
            const base64 = await $history.readImage({
              filename: historyItem.content,
            });
            const size = Math.ceil((base64.length * 3) / 4);
            imageSizes.value[historyItem.id] = formatFileSize(size);

            const img = new Image();
            img.src = `data:image/png;base64,${base64}`;
            imageUrls.value[historyItem.id] = img.src;

            await new Promise<void>((resolveProm) => {
              img.onload = () => {
                imageDimensions.value[
                  historyItem.id
                ] = `${img.width}x${img.height}`;
                resolveProm();
              };
              img.onerror = () => {
                imageDimensions.value[historyItem.id] = "Error";
                resolveProm();
              };
            });
          } catch (error) {
            console.error("Error processing image:", error);
            imageDimensions.value[historyItem.id] = "Error";
            imageSizes.value[historyItem.id] = "Error";
          }
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

const scrollToSelectedItem = (): void => {
  nextTick(() => {
    const viewport = resultsContainer.value?.osInstance()?.elements().viewport;
    if (!selectedElement.value || !viewport) return;

    setTimeout(() => {
      if (!selectedElement.value) return;

      const viewportRect = viewport.getBoundingClientRect();
      const elementRect = selectedElement.value.getBoundingClientRect();

      const isFirstItemInGroup = selectedItemIndex.value === 0;
      const isAbove = elementRect.top < viewportRect.top + SCROLL_PADDING;
      const isBelow = elementRect.bottom > viewportRect.bottom - SCROLL_PADDING;

      if (isAbove) {
        viewport.scrollTo({
          top:
            viewport.scrollTop +
            (elementRect.top - viewportRect.top) -
            (isFirstItemInGroup ? TOP_SCROLL_PADDING : SCROLL_PADDING),
          behavior: "smooth",
        });
      } else if (isBelow) {
        viewport.scrollTo({
          top:
            viewport.scrollTop +
            (elementRect.bottom - viewportRect.bottom) +
            SCROLL_PADDING,
          behavior: "smooth",
        });
      }
    }, 10);
  });
};

let searchController: AbortController | null = null;
let searchQueue: Array<string> = [];
let isProcessingSearch = false;

const searchStarted = () => {
  if (searchController) {
    searchController.abort();
  }
};

const processSearchQueue = async () => {
  if (isProcessingSearch || searchQueue.length === 0) return;

  isProcessingSearch = true;
  const query = searchQueue.pop();
  searchQueue = [];

  try {
    if (!query || !query.trim()) {
      history.value = [];
      offset = 0;
      await loadHistoryChunk();
      isProcessingSearch = false;
      return;
    }

    const results = await $history.searchHistory(query);

    if (searchController?.signal.aborted) {
      isProcessingSearch = false;
      return;
    }

    history.value = results.map((item) =>
      Object.assign(
        new HistoryItem(
          item.source,
          item.content_type,
          item.content,
          item.favicon,
          item.source_icon,
          item.language
        ),
        { id: item.id, timestamp: new Date(item.timestamp) }
      )
    );
    if (groupedHistory.value.length > 0) {
      handleSelection(0, 0, true);
    } else {
      selectItem(-1, -1);
    }
  } catch (error) {
    console.error("Search error:", error);
  } finally {
    isProcessingSearch = false;
    if (searchQueue.length > 0) {
      requestAnimationFrame(() => processSearchQueue());
    }
  }
};

const searchHistory = async (query: string): Promise<void> => {
  searchQuery.value = query;

  if (searchController) {
    searchController.abort();
  }

  searchController = new AbortController();

  searchQueue.push(query);
  if (!isProcessingSearch) {
    processSearchQueue();
  }
};

watch(
  () => groupedHistory.value,
  (newGroupedHistory, oldGroupedHistory) => {
    if (
      newGroupedHistory.length > 0 &&
      oldGroupedHistory &&
      oldGroupedHistory.length === 0
    ) {
      handleSelection(0, 0, true);
    } else if (newGroupedHistory.length === 0) {
      selectItem(-1, -1);
    }
  },
  { deep: true }
);

const pasteSelectedItem = async (): Promise<void> => {
  if (!selectedItem.value) return;

  let content = selectedItem.value.content;
  let contentType: string = selectedItem.value.content_type;
  if (contentType === "image") {
    try {
      content = await $history.readImage({ filename: content });
    } catch (error) {
      console.error("Error reading image file:", error);
      return;
    }
  }
  await hideApp();
  await $history.writeAndPaste({ content, contentType });
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
    ? `https://img.youtube.com/vi/${videoId}/maxresdefault.jpg`
    : "https://via.placeholder.com/1280x720";
};

const updateHistory = async (resetScroll: boolean = false): Promise<void> => {
  offset = 0;
  history.value = [];

  const results = await $history.loadHistoryChunk(offset, CHUNK_SIZE);
  if (results.length > 0) {
    const processedItems = await Promise.all(
      results.map(async (item) => {
        const historyItem = new HistoryItem(
          item.source,
          item.content_type,
          item.content,
          item.favicon,
          item.source_icon,
          item.language
        );
        Object.assign(historyItem, {
          id: item.id,
          timestamp: new Date(item.timestamp),
        });

        if (historyItem.content_type === ContentType.Image) {
          try {
            const base64 = await $history.readImage({
              filename: historyItem.content,
            });
            const size = Math.ceil((base64.length * 3) / 4);
            imageSizes.value[historyItem.id] = formatFileSize(size);

            const img = new Image();
            img.src = `data:image/png;base64,${base64}`;
            imageUrls.value[historyItem.id] = img.src;

            await new Promise<void>((resolveProm) => {
              img.onload = () => {
                imageDimensions.value[
                  historyItem.id
                ] = `${img.width}x${img.height}`;
                resolveProm();
              };
              img.onerror = () => {
                imageDimensions.value[historyItem.id] = "Error";
                resolveProm();
              };
            });
          } catch (error) {
            console.error("Error processing image:", error);
            imageDimensions.value[historyItem.id] = "Error";
            imageSizes.value[historyItem.id] = "Error";
          }
        }
        return historyItem;
      })
    );

    history.value = processedItems;
    offset = results.length;

    if (
      resetScroll &&
      resultsContainer.value?.osInstance()?.elements().viewport
    ) {
      resultsContainer.value.osInstance()?.elements().viewport?.scrollTo({
        top: 0,
        behavior: "smooth",
      });
    }
  }
};

const handleSelection = (
  groupIndex: number,
  itemIndex: number,
  shouldScroll: boolean = true
): void => {
  selectItem(groupIndex, itemIndex);
  if (shouldScroll) scrollToSelectedItem();
};

const { hideApp } = useAppControl();

const focusSearchInput = (): void => {
  nextTick(() => {
    topBar.value?.searchInput?.focus();
  });
};

const onImageError = (): void => {
  imageLoadError.value = true;
  imageLoading.value = false;
};

watch(
  [selectedGroupIndex, selectedItemIndex],
  () => {
    scrollToSelectedItem();
  },
  { flush: "post" }
);

onMounted(async () => {
  try {
    await loadHistoryChunk();
    if (groupedHistory.value.length > 0 && !selectedItem.value) {
      handleSelection(0, 0, true);
    }

    resultsContainer.value
      ?.osInstance()
      ?.elements()
      ?.viewport?.addEventListener("scroll", handleScroll);

    listeners.push(
      await listen("clipboard-content-updated", async () => {
        lastUpdateTime.value = Date.now();
        await updateHistory(true);
        if (groupedHistory.value[0]?.items.length > 0) {
          handleSelection(0, 0, false);
        }
      })
    );

    listeners.push(
      await listen("tauri://focus", async () => {
        console.log("Tauri window focused");
        // Attempt to re-initialize keyboard listeners
        if ($keyboard && typeof $keyboard.init === "function") {
          console.log("Re-initializing keyboard via $keyboard.init()");
          $keyboard.init();
        } else {
          console.warn("$keyboard.init is not available");
        }

        const currentTime = Date.now();
        if (currentTime - lastUpdateTime.value > 0) {
          const previousState = {
            groupIndex: selectedGroupIndex.value,
            itemIndex: selectedItemIndex.value,
            scroll:
              resultsContainer.value?.osInstance()?.elements().viewport
                ?.scrollTop || 0,
          };

          await updateHistory();
          lastUpdateTime.value = currentTime;
          handleSelection(
            previousState.groupIndex,
            previousState.itemIndex,
            false
          );

          if (
            resultsContainer.value?.osInstance()?.elements().viewport?.scrollTo
          ) {
            resultsContainer.value.osInstance()?.elements().viewport?.scrollTo({
              top: previousState.scroll,
              behavior: "instant",
            });
          }
        }
        focusSearchInput();
      })
    );

    listeners.push(
      await listen("tauri://blur", () => {
        searchInput.value?.blur();
      })
    );

    listeners.push(
      $keyboard.listen(
        [$keyboard.Key.DownArrow],
        () => {
          console.log(
            "Down Arrow pressed. Active element:",
            document.activeElement
          );
          selectNext();
        },
        { prevent: true }
      )
    );
    listeners.push(
      $keyboard.listen(
        [$keyboard.Key.UpArrow],
        () => {
          console.log(
            "Up Arrow pressed. Active element:",
            document.activeElement
          );
          selectPrevious();
        },
        { prevent: true }
      )
    );
    listeners.push(
      $keyboard.listen([$keyboard.Key.Enter], pasteSelectedItem, {
        prevent: true,
      })
    );
    listeners.push(
      $keyboard.listen(
        [$keyboard.Key.Escape],
        () => {
          if (isActionsMenuVisible.value) {
            closeActionsMenu();
          } else {
            hideApp();
          }
        },
        { prevent: true }
      )
    );

    const metaOrCtrlKey =
      $keyboard.currentOS === "macos"
        ? $keyboard.Key.Meta
        : $keyboard.Key.Control;
    listeners.push(
      $keyboard.listen([metaOrCtrlKey, $keyboard.Key.K], toggleActionsMenu, {
        prevent: true,
        ignoreIfEditable: true,
      })
    );
  } catch (error) {
    console.error("Error during onMounted:", error);
  }
});

onUnmounted(() => {
  listeners.forEach((unlisten) => {
    if (typeof unlisten === "function") {
      unlisten();
    }
  });
  listeners.length = 0;
  const viewport = resultsContainer.value?.osInstance()?.elements()?.viewport;
  if (viewport) {
    viewport.removeEventListener("scroll", handleScroll);
  }
});

const getFormattedDate = computed(() => {
  if (!selectedItem.value?.timestamp) return "";
  return new Intl.DateTimeFormat("en-US", {
    dateStyle: "medium",
    timeStyle: "medium",
  }).format(selectedItem.value.timestamp);
});

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

const fetchPageMeta = async (url: string) => {
  try {
    const [title, ogImage] = (await invoke("fetch_page_meta", { url })) as [
      string,
      string | null
    ];
    pageTitle.value = title;
    if (ogImage) {
      pageOgImage.value = ogImage;
    }
  } catch (error) {
    console.error("Error fetching page meta:", error);
    pageTitle.value = "Error loading title";
  }
};

watch(
  () => selectedItem.value,
  (newItem) => {
    if (newItem?.content_type === ContentType.Link) {
      pageTitle.value = "Loading...";
      pageOgImage.value = "";
      fetchPageMeta(newItem.content);
    } else {
      pageTitle.value = "";
      pageOgImage.value = "";
    }
  }
);

const getInfo = computed(() => {
  if (!selectedItem.value) return null;

  const baseInfo = {
    source: selectedItem.value.source,
    copied: selectedItem.value.timestamp,
  };

  const infoMap: Record<
    ContentType,
    () => InfoText | InfoImage | InfoFile | InfoLink | InfoColor | InfoCode
  > = {
    [ContentType.Text]: () => ({
      ...baseInfo,
      content_type: ContentType.Text,
      characters: selectedItem.value!.content.length,
      words: selectedItem.value!.content.trim().split(/\s+/).length,
    }),
    [ContentType.Image]: () => ({
      ...baseInfo,
      content_type: ContentType.Image,
      dimensions: imageDimensions.value[selectedItem.value!.id] || "Loading...",
      size: parseInt(imageSizes.value[selectedItem.value!.id] || "0"),
    }),
    [ContentType.File]: () => ({
      ...baseInfo,
      content_type: ContentType.File,
      path: selectedItem.value!.content,
      filesize: 0,
    }),
    [ContentType.Link]: () => ({
      ...baseInfo,
      content_type: ContentType.Link,
      title: pageTitle.value,
      url: selectedItem.value!.content,
      characters: selectedItem.value!.content.length,
    }),
    [ContentType.Color]: () => {
      const hex = selectedItem.value!.content;
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);

      const rNorm = r / 255;
      const gNorm = g / 255;
      const bNorm = b / 255;

      const max = Math.max(rNorm, gNorm, bNorm);
      const min = Math.min(rNorm, gNorm, bNorm);
      let h = 0,
        s = 0;
      const l = (max + min) / 2;

      if (max !== min) {
        const d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

        switch (max) {
          case rNorm:
            h = (gNorm - bNorm) / d + (gNorm < bNorm ? 6 : 0);
            break;
          case gNorm:
            h = (bNorm - rNorm) / d + 2;
            break;
          case bNorm:
            h = (rNorm - gNorm) / d + 4;
            break;
        }
        h /= 6;
      }

      return {
        ...baseInfo,
        content_type: ContentType.Color,
        hex: hex,
        rgb: `rgb(${r}, ${g}, ${b})`,
        hsl: `hsl(${Math.round(h * 360)}, ${Math.round(s * 100)}%, ${Math.round(
          l * 100
        )}%)`,
      };
    },
    [ContentType.Code]: () => ({
      ...baseInfo,
      content_type: ContentType.Code,
      language: selectedItem.value!.language ?? "Unknown",
      lines: selectedItem.value!.content.split("\n").length,
    }),
  };

  return infoMap[selectedItem.value.content_type]();
});

const infoRows = computed(() => {
  if (!getInfo.value) return [];

  const commonRows = [
    {
      label: "Source",
      value: getInfo.value.source,
      isUrl: false,
      icon: selectedItem.value?.source_icon
        ? `data:image/png;base64,${selectedItem.value.source_icon}`
        : undefined,
    },
    {
      label: "Content Type",
      value:
        getInfo.value.content_type.charAt(0).toUpperCase() +
        getInfo.value.content_type.slice(1),
      isUrl: false,
    },
  ];

  const typeSpecificRows: Record<
    ContentType,
    Array<{
      label: string;
      value: string | number;
      isUrl?: boolean;
      icon?: string;
    }>
  > = {
    [ContentType.Text]: [
      { label: "Characters", value: (getInfo.value as InfoText).characters },
      { label: "Words", value: (getInfo.value as InfoText).words },
    ],
    [ContentType.Image]: [
      { label: "Dimensions", value: (getInfo.value as InfoImage).dimensions },
      {
        label: "Image size",
        value: formatFileSize((getInfo.value as InfoImage).size),
      },
    ],
    [ContentType.File]: [
      { label: "Path", value: (getInfo.value as InfoFile).path },
    ],
    [ContentType.Link]: [
      ...((getInfo.value as InfoLink).title &&
      (getInfo.value as InfoLink).title !== "Loading..."
        ? [{ label: "Title", value: (getInfo.value as InfoLink).title || "" }]
        : []),
      { label: "URL", value: (getInfo.value as InfoLink).url, isUrl: true },
      { label: "Characters", value: (getInfo.value as InfoLink).characters },
    ],
    [ContentType.Color]: [
      { label: "Hex", value: (getInfo.value as InfoColor).hex },
      { label: "RGB", value: (getInfo.value as InfoColor).rgb },
      { label: "HSL", value: (getInfo.value as InfoColor).hsl },
    ],
    [ContentType.Code]: [
      { label: "Language", value: (getInfo.value as InfoCode).language },
      { label: "Lines", value: (getInfo.value as InfoCode).lines },
    ],
  };

  const specificRows = typeSpecificRows[getInfo.value.content_type].filter(
    (row) => row.value !== ""
  );

  return [
    ...commonRows,
    ...specificRows,
    { label: "Copied", value: getFormattedDate.value },
  ];
});
</script>

<style scoped lang="scss">
@use "/styles/index.scss";
</style>
