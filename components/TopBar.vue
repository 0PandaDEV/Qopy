<template>
  <div class="topbar">
    <input
      ref="searchInput"
      v-model="searchQuery"
      @input="onSearch"
      class="search"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      type="text"
      placeholder="Type to filter entries..." />
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

const searchQuery = ref("");
const searchInput = ref<HTMLInputElement | null>(null);

const emit = defineEmits<{
  (e: "search", query: string): void;
  (e: "focus"): void;
}>();

const onSearch = () => {
  emit("search", searchQuery.value);
};

defineExpose({ searchInput });
</script>

<style lang="scss">
.topbar {
  width: 100%;
  min-height: 56px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  padding-inline: 16px;
  z-index: 100;

  .search {
    width: 100%;
    height: 100%;
    font-size: 18px;
    color: var(--text);
    background-color: transparent;
    outline: none;
    border: none;
    font-family: SFRoundedMedium;
  }
}
</style>
