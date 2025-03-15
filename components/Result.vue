<template>
  <div
    :class="['result', { selected }]"
    @click="$emit('select')"
    :ref="el => { if (selected && el) $emit('setRef', el as HTMLElement) }">
    <template v-if="item.content_type === 'image'">
      <img
        v-if="imageUrl"
        :src="imageUrl"
        alt="Image"
        class="image"
        @error="$emit('imageError')" />
      <IconsImage v-else class="icon" />
    </template>
    <template v-else-if="hasFavicon(item.favicon ?? '')">
      <img
        v-if="item.favicon"
        :src="getFaviconFromDb(item.favicon)"
        alt="Favicon"
        class="favicon"
        @error="
          ($event.target as HTMLImageElement).src = '/public/icons/Link.svg'
        " />
      <IconsLink v-else class="icon" />
    </template>
    <IconsFile
      class="icon"
      v-else-if="item.content_type === ContentType.File" />
    <IconsText
      class="icon"
      v-else-if="item.content_type === ContentType.Text" />
    <svg
      v-else-if="item.content_type === ContentType.Color"
      width="18"
      height="18"
      viewBox="0 0 18 18"
      fill="none"
      xmlns="http://www.w3.org/2000/svg">
      <g>
        <rect width="18" height="18" />
        <path
          d="M9 18C12.2154 18 15.1865 16.2846 16.7942 13.5C18.4019 10.7154 18.4019 7.28461 16.7942 4.5C15.1865 1.71539 12.2154 -1.22615e-06 9 0C5.78461 0 2.81347 1.71539 1.20577 4.5C-0.401925 7.28461 -0.401923 10.7154 1.20577 13.5C2.81347 16.2846 5.78461 18 9 18Z"
          fill="#E5DFD5" />
        <path
          d="M9 16C7.14348 16 5.36301 15.2625 4.05025 13.9497C2.7375 12.637 2 10.8565 2 9C2 7.14348 2.7375 5.36301 4.05025 4.05025C5.36301 2.7375 7.14348 2 9 2C10.8565 2 12.637 2.7375 13.9497 4.05025C15.2625 5.36301 16 7.14348 16 9C16 10.8565 15.2625 12.637 13.9497 13.9497C12.637 15.2625 10.8565 16 9 16Z"
          :fill="item.content" />
      </g>
    </svg>
    <IconsCode
      class="icon"
      v-else-if="item.content_type === ContentType.Code" />
    <span v-if="item.content_type === ContentType.Image">
      Image ({{ dimensions || "Loading..." }})
    </span>
    <span v-else>{{ truncateContent(item.content) }}</span>
  </div>
</template>

<script setup lang="ts">
import { ContentType } from "~/types/types";
import type { HistoryItem } from "~/types/types";

defineProps<{
  item: HistoryItem;
  selected: boolean;
  imageUrl?: string;
  dimensions?: string;
}>();

defineEmits<{
  (e: "select"): void;
  (e: "imageError"): void;
  (e: "setRef", el: HTMLElement): void;
}>();

const hasFavicon = (str: string): boolean => {
  return str.trim() !== "";
};

const getFaviconFromDb = (favicon: string): string => {
  return `data:image/png;base64,${favicon}`;
};

const truncateContent = (content: string): string => {
  const maxWidth = 284;
  const charWidth = 9;
  const maxChars = Math.floor(maxWidth / charWidth);
  return content.length > maxChars
    ? content.slice(0, maxChars - 3) + "..."
    : content;
};
</script>

<style scoped lang="scss">
.result {
  display: flex;
  gap: 12px;
  padding: 11px;
  border-radius: 10px;
  cursor: pointer;
      align-items: center;

  &.selected {
    background-color: var(--border);
  }

  .favicon,
  .image,
  .icon {
    width: 18px;
    height: 18px;
  }

  span {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text);
  }
}
</style>
