<template>
  <div v-if="isVisible" class="actions" ref="menuRef">
    <OverlayScrollbarsComponent ref="scrollbarsRef" class="actions-scrollable"
      :options="{ scrollbars: { autoHide: 'scroll' } }">
      <template v-if="searchQuery">
        <div class="action-group">
          <div v-if="allFilteredActions.length === 0" class="action no-results">
            <div class="content">
              <div class="title">No Results</div>
            </div>
          </div>
          <div v-else v-for="(action, index) in allFilteredActions" :key="action.action" class="action"
            @click="executeAction(action)" :class="{ selected: isSelected && currentIndex === index }" :ref="(el) => {
              if (currentIndex === index) setSelectedElement(el);
            }
              " :style="action.color ? { color: action.color } : {}">
            <div class="content">
              <component v-if="action.icon" :is="action.icon" class="icon" />
              <div class="title">{{ action.title }}</div>
            </div>
            <div v-if="action.shortcut" class="shortcut">
              <template v-for="(key, keyIndex) in parseShortcut(action.shortcut)" :key="keyIndex">
                <component :is="key.component" v-if="key.component" :input="key.value" />
              </template>
            </div>
          </div>
        </div>
      </template>

      <template v-else>
        <div class="action-group">
          <div v-for="(action, index) in topActions" :key="action.action" class="action" @click="executeAction(action)"
            :class="{
              selected:
                isSelected && currentIndex === getActionIndex(index, 'top'),
            }" :ref="(el) => {
              if (currentIndex === getActionIndex(index, 'top'))
                setSelectedElement(el);
            }
              ">
            <div class="content">
              <component v-if="action.icon" :is="action.icon" class="icon" />
              <div class="title">{{ action.title }}</div>
            </div>
            <div v-if="action.shortcut" class="shortcut">
              <template v-for="(key, index) in parseShortcut(action.shortcut)" :key="index">
                <component :is="key.component" v-if="key.component" :input="key.value" />
              </template>
            </div>
          </div>
          <div class="divider" v-if="
            topActions.length > 0 && typeSpecificActions.length > 0
          "></div>
        </div>

        <div v-if="typeSpecificActions.length > 0" class="action-group">
          <div v-for="(action, index) in typeSpecificActions" :key="action.action" class="action"
            @click="executeAction(action)" :class="{
              selected:
                isSelected &&
                currentIndex === getActionIndex(index, 'specific'),
            }" :ref="(el) => {
              if (currentIndex === getActionIndex(index, 'specific'))
                setSelectedElement(el);
            }
              ">
            <div class="content">
              <component v-if="action.icon" :is="action.icon" class="icon" />
              <div class="title">{{ action.title }}</div>
            </div>
            <div v-if="action.shortcut" class="shortcut">
              <template v-for="(key, index) in parseShortcut(action.shortcut)" :key="index">
                <component :is="key.component" v-if="key.component" :input="key.value" />
              </template>
            </div>
          </div>
          <div class="divider" v-if="
            typeSpecificActions.length > 0 && bottomActions.length > 0
          "></div>
        </div>

        <div class="action-group">
          <div v-for="(action, index) in bottomActions" :key="action.action" class="action"
            @click="executeAction(action)" :class="{
              selected:
                isSelected && currentIndex === getActionIndex(index, 'bottom'),
            }" :ref="(el) => {
              if (currentIndex === getActionIndex(index, 'bottom'))
                setSelectedElement(el);
            }
              " :style="action.color ? { color: action.color } : {}">
            <div class="content">
              <component v-if="action.icon" :is="action.icon" class="icon" />
              <div class="title">{{ action.title }}</div>
            </div>
            <div v-if="action.shortcut" class="shortcut">
              <template v-for="(key, index) in parseShortcut(action.shortcut)" :key="index">
                <component :is="key.component" v-if="key.component" :input="key.value" />
              </template>
            </div>
          </div>
        </div>
      </template>
    </OverlayScrollbarsComponent>

    <input type="text" v-model="searchQuery" class="search-input" placeholder="Search..." @keydown="handleSearchKeydown"
      ref="searchInput" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick, h } from "vue";
import { OverlayScrollbarsComponent } from "overlayscrollbars-vue";
import "overlayscrollbars/overlayscrollbars.css";
import Enter from "./Keys/Enter.vue";
import Cmd from "./Keys/Cmd.vue";
import Key from "./Keys/Key.vue";
import { ContentType, HistoryItem } from "../types/types";
import { invoke } from "@tauri-apps/api/core";
import { useNuxtApp } from "#app";
import Shift from "./Keys/Shift.vue";
import Gear from "./Icons/Gear.vue";
import Bin from "./Icons/Bin.vue";
import Pen from "./Icons/Pen.vue";
import T from "./Icons/T.vue";
import Board from "./Icons/Board.vue";
import Open from "./Icons/Open.vue";
import Globe from "./Icons/Globe.vue";
import Zip from "./Icons/Zip.vue";
import Brush from "./Icons/Brush.vue";
import Rotate from "./Icons/Rotate.vue";
import Expand from "./Icons/Expand.vue";
import { useActions } from "../composables/useActions";

interface AppInfo {
  name: string;
  icon?: string;
}

const currentAppInfo = ref<AppInfo>({ name: "Current App" });
const isSelected = ref(true);
const currentIndex = ref(0);
const selectedElement = ref<HTMLElement | null>(null);
const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const { $keyboard } = useNuxtApp();
const menuRef = ref<HTMLElement | null>(null);
const scrollbarsRef = ref<InstanceType<
  typeof OverlayScrollbarsComponent
> | null>(null);
const { handleAction } = useActions();

const SCROLL_PADDING = 8;

const setSelectedElement = (el: any) => {
  if (el && el instanceof HTMLElement) {
    selectedElement.value = el;
  }
};

const getAppInfo = async () => {
  try {
    const appInfo = await invoke("get_app_info");
    if (appInfo && typeof appInfo === "object" && "name" in appInfo) {
      currentAppInfo.value = appInfo as AppInfo;
    }
  } catch (error) {
    console.error("Failed to get app info:", error);
  }
};

const props = defineProps<{
  isVisible: boolean;
  selectedItem?: HistoryItem;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "toggle"): void;
  (e: "action", action: string, item?: HistoryItem): void;
}>();

interface ActionItem {
  title: string;
  action: string;
  shortcut?: string;
  icon?: any;
  group: string;
  color?: string;
}

const topActions = computed((): ActionItem[] => [
  {
    title: `Paste to ${currentAppInfo.value.name || "Current App"}`,
    shortcut: "Enter",
    action: "paste-to-app",
    group: "top",
    icon: currentAppInfo.value.icon ? {
      render() {
        return h('img', {
          src: currentAppInfo.value.icon,
          style: {
            width: '14px',
            height: '14px',
            objectFit: 'contain'
          }
        });
      }
    } : undefined,
  },
  {
    title: "Copy to Clipboard",
    shortcut: "Ctrl+C",
    action: "copy",
    group: "top",
    icon: Board,
  },
]);

const bottomActions = computed((): ActionItem[] => [
  {
    title: "Delete Entry",
    shortcut: "Alt+X",
    action: "delete",
    group: "bottom",
    icon: Bin,
    color: "var(--red)"
  },
  {
    title: "Delete All Entries",
    shortcut: "Alt+Shift+X",
    action: "delete-all",
    group: "bottom",
    icon: Bin,
    color: "var(--red)"
  },
  {
    title: "Settings",
    shortcut: "Ctrl+S",
    action: "settings",
    group: "bottom",
    icon: Gear,
  },
]);

const textActions = computed((): ActionItem[] => [
  {
    title: "Paste as plain text",
    action: "paste-plain",
    shortcut: "Ctrl+Shift+V",
    group: "text",
    icon: T,
  },
  {
    title: "Edit text",
    action: "edit-text",
    shortcut: "Ctrl+E",
    group: "text",
    icon: Pen,
  },
]);

const imageActions = computed((): ActionItem[] => [
  {
    title: "Rotate",
    action: "rotate-image",
    shortcut: "Alt+R",
    group: "image",
    icon: Rotate,
  },
  {
    title: "Resize",
    action: "resize-image",
    shortcut: "Alt+S",
    group: "image",
    icon: Expand,
  },
  {
    title: "Compress",
    action: "compress-image",
    shortcut: "Alt+C",
    group: "image",
    icon: Zip,
  },
]);

const fileActions = computed((): ActionItem[] => [
  {
    title: "Open",
    action: "open-file",
    shortcut: "Ctrl+O",
    group: "file",
    icon: Open,
  },
  {
    title: "Compress to zip",
    action: "compress-file",
    shortcut: "Alt+C",
    group: "file",
    icon: Zip,
  },
]);

const linkActions = computed((): ActionItem[] => [
  {
    title: "Open in Browser",
    action: "open-link",
    shortcut: "Ctrl+O",
    group: "link",
    icon: Globe,
  },
]);

const colorActions = computed((): ActionItem[] => [
  {
    title: "Copy as HEX",
    action: "copy-hex",
    shortcut: "Alt+H",
    group: "color",
    icon: Brush,
  },
  {
    title: "Copy as RGB(a)",
    action: "copy-rgba",
    shortcut: "Alt+R",
    group: "color",
    icon: Brush,
  },
  {
    title: "Copy as HSL(a)",
    action: "copy-hsla",
    shortcut: "Alt+S",
    group: "color",
    icon: Brush,
  },
]);

const typeSpecificActions = computed(() => {
  if (!props.selectedItem) return [];

  switch (props.selectedItem.content_type) {
    case ContentType.Text:
      return textActions.value;
    case ContentType.Image:
      return imageActions.value;
    case ContentType.File:
      return fileActions.value;
    case ContentType.Link:
      return linkActions.value;
    case ContentType.Color:
      return colorActions.value;
    case ContentType.Code:
      return textActions.value;
    default:
      return [];
  }
});

const allActions = computed(() => {
  return [
    ...topActions.value,
    ...typeSpecificActions.value,
    ...bottomActions.value,
  ];
});

const allFilteredActions = computed(() => {
  if (!searchQuery.value) return allActions.value;

  return allActions.value.filter((action) =>
    action.title.toLowerCase().includes(searchQuery.value.toLowerCase())
  );
});

const getActionIndex = (
  index: number,
  group: "top" | "specific" | "bottom"
): number => {
  if (group === "top") {
    return index;
  } else if (group === "specific") {
    return topActions.value.length + index;
  } else {
    return topActions.value.length + typeSpecificActions.value.length + index;
  }
};

interface KeyPart {
  type: "modifier" | "key" | "separator";
  value: string;
  component?: any;
}

const parseShortcut = (shortcut: string): KeyPart[] => {
  const parts = shortcut.split("+");
  const result: KeyPart[] = [];

  parts.forEach((part, index) => {
    const trimmedPart = part.trim();
    let keyPart: KeyPart;

    if (trimmedPart.toLowerCase() === "cmd") {
      keyPart = { type: "modifier", value: trimmedPart, component: Cmd };
    } else if (trimmedPart.toLowerCase() === "shift") {
      keyPart = { type: "modifier", value: trimmedPart, component: Shift };
    } else if (trimmedPart.toLowerCase() === "enter") {
      keyPart = { type: "key", value: trimmedPart, component: Enter };
    } else {
      keyPart = { type: "key", value: trimmedPart, component: Key };
    }

    result.push(keyPart);

    if (index < parts.length - 1) {
      result.push({ type: "separator", value: "+" });
    }
  });

  return result;
};

const executeAction = (action: ActionItem) => {
  emit("close");
  handleAction(action.action, props.selectedItem);
};

const close = () => {
  emit("close");
};

const handleClickOutside = (event: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    close();
  }
};

const handleWindowBlur = () => {
  close();
};

const setupKeyboardHandlers = () => {
  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.ArrowDown],
    (event) => {
      event.preventDefault();
      selectNext();
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );

  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.ArrowUp],
    (event) => {
      event.preventDefault();
      selectPrevious();
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );

  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.Enter],
    (event) => {
      event.preventDefault();
      if (searchQuery.value) {
        const action = allFilteredActions.value[currentIndex.value];
        if (action) executeAction(action);
      } else {
        let action;
        if (currentIndex.value < topActions.value.length) {
          action = topActions.value[currentIndex.value];
        } else if (
          currentIndex.value <
          topActions.value.length + typeSpecificActions.value.length
        ) {
          action =
            typeSpecificActions.value[
            currentIndex.value - topActions.value.length
            ];
        } else {
          action =
            bottomActions.value[
            currentIndex.value -
            topActions.value.length -
            typeSpecificActions.value.length
            ];
        }
        if (action) executeAction(action);
      }
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );

  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.Escape],
    (event) => {
      event.preventDefault();
      close();
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );
  
  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.LeftControl, $keyboard.Key.K],
    (event) => {
      event.preventDefault();
      emit("toggle");
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );
  
  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.RightControl, $keyboard.Key.K],
    (event) => {
      event.preventDefault();
      emit("toggle");
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );
  
  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.MetaLeft, $keyboard.Key.K],
    (event) => {
      event.preventDefault();
      emit("toggle");
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );
  
  $keyboard.on(
    "actionsMenu",
    [$keyboard.Key.MetaRight, $keyboard.Key.K],
    (event) => {
      event.preventDefault();
      emit("toggle");
    },
    { priority: $keyboard.PRIORITY.HIGH }
  );
};

const selectNext = () => {
  if (searchQuery.value) {
    if (allFilteredActions.value.length === 0) return;
    currentIndex.value =
      (currentIndex.value + 1) % allFilteredActions.value.length;
  } else {
    const totalActions = allActions.value.length;
    if (totalActions === 0) return;
    currentIndex.value = (currentIndex.value + 1) % totalActions;
  }
  scrollToSelected();
};

const selectPrevious = () => {
  if (searchQuery.value) {
    if (allFilteredActions.value.length === 0) return;
    currentIndex.value =
      (currentIndex.value - 1 + allFilteredActions.value.length) %
      allFilteredActions.value.length;
  } else {
    const totalActions = allActions.value.length;
    if (totalActions === 0) return;
    currentIndex.value = (currentIndex.value - 1 + totalActions) % totalActions;
  }
  scrollToSelected();
};

const scrollToSelected = () => {
  nextTick(() => {
    if (!selectedElement.value) return;
    if (!scrollbarsRef.value) return;

    const viewport = scrollbarsRef.value.osInstance()?.elements().viewport;
    if (!viewport) {
      selectedElement.value.scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
      return;
    }

    setTimeout(() => {
      if (!selectedElement.value) return;

      const viewportRect = viewport.getBoundingClientRect();
      const elementRect = selectedElement.value.getBoundingClientRect();

      const isAbove = elementRect.top < viewportRect.top + SCROLL_PADDING;
      const isBelow = elementRect.bottom > viewportRect.bottom - SCROLL_PADDING;

      if (isAbove) {
        const scrollAmount =
          viewport.scrollTop +
          (elementRect.top - viewportRect.top) -
          SCROLL_PADDING;
        viewport.scrollTo({
          top: scrollAmount,
          behavior: "smooth",
        });
      } else if (isBelow) {
        const scrollAmount =
          viewport.scrollTop +
          (elementRect.bottom - viewportRect.bottom) +
          SCROLL_PADDING;
        viewport.scrollTo({
          top: scrollAmount,
          behavior: "smooth",
        });
      }
    }, 10);
  });
};

const handleSearchKeydown = (event: KeyboardEvent) => {
  if (
    event.key === "ArrowDown" ||
    event.key === "ArrowUp" ||
    event.key === "Enter" ||
    event.key === "Escape"
  ) {
    return;
  }
  
  if (event.key.toLowerCase() === "k" && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    event.stopPropagation();
    emit("toggle");
    return;
  }

  event.stopPropagation();
};

watch(
  () => props.isVisible,
  (visible) => {
    if (visible) {
      currentIndex.value = 0;
      searchQuery.value = "";
      setupKeyboardHandlers();
      $keyboard.enableContext("actionsMenu");

      nextTick(() => {
        if (searchInput.value) {
          setTimeout(() => {
            searchInput.value?.focus();
          }, 50);
        }
      });
    } else {
      $keyboard.disableContext("actionsMenu");
    }
  }
);

watch(searchQuery, (query) => {
  currentIndex.value = 0;
});

watch(
  [allFilteredActions, topActions, typeSpecificActions, bottomActions],
  () => {
    if (searchQuery.value) {
      if (
        currentIndex.value >= allFilteredActions.value.length &&
        allFilteredActions.value.length > 0
      ) {
        currentIndex.value = 0;
      }
    } else {
      const totalActions = allActions.value.length;
      if (currentIndex.value >= totalActions && totalActions > 0) {
        currentIndex.value = 0;
      }
    }
  }
);

onMounted(() => {
  document.addEventListener("click", handleClickOutside);
  window.addEventListener("blur", handleWindowBlur);
  getAppInfo();
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickOutside);
  window.removeEventListener("blur", handleWindowBlur);
  $keyboard.disableContext("actionsMenu");
});
</script>

<style scoped lang="scss">
.actions {
  border-radius: 8px;
  border: 1px solid var(--border);
  background-color: var(--background);
  position: fixed;
  bottom: 48px;
  right: 8px;
  z-index: 100;
  width: 350px;
  max-height: 250px;
  height: auto;
  display: flex;
  flex-direction: column;
  color: var(--text);
}

.actions-scrollable {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.action-group {
  display: flex;
  flex-direction: column;
  padding-inline: 8px;
}

.divider {
  height: 1px;
  background-color: var(--border);
  margin: 8px -8px;
  width: calc(100% + 16px);
}

.action.no-results {
  justify-content: center;
  text-align: center;
  color: var(--text-secondary);
  width: 100%;
}

.action {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-inline: 8px;
  height: 36px;
  cursor: pointer;
  border-radius: 5px;

  &.selected {
    background-color: var(--border);
  }

  .shortcut {
    display: flex;
    gap: 2px;
    height: 20px;
  }

  .content {
    display: flex;
    align-items: center;
    gap: 8px;

    .icon {
      width: 14px;
      height: 14px;
    }

    .title {
      color: inherit;
    }
  }
}

.search-input {
  width: 100%;
  padding: 12px 16px;
  outline: none;
  border: none;
  border-top: 1px solid var(--border);
  background-color: var(--background);
  color: var(--text);
  font-size: 14px;
  border-bottom-left-radius: 7px;
  border-bottom-right-radius: 7px;
  margin-top: auto;
}
</style>
