<template>
  <div class="bg">
    <div class="keybind-container">
      <h2>Set New Keybind</h2>
      <div
        class="keybind-input"
        tabindex="0"
        @focus="startCapture"
        @blur="stopCapture"
        ref="keybindInput"
      >
        {{ currentKeybind || 'Click here, then press your desired key combination' }}
      </div>
      <button @click="saveKeybind" :disabled="!currentKeybind">Save Keybind</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const currentKeybind = ref('');
const keybindInput = ref<HTMLElement | null>(null);

const startCapture = async () => {
  await invoke('start_keybind_capture');
};

const stopCapture = async () => {
  await invoke('stop_keybind_capture');
};

const saveKeybind = () => {
  console.log('Saving keybind:', currentKeybind.value);
  // Implement saving logic here
};

onMounted(async () => {
  const unlisten = await listen('keybind_captured', (event: any) => {
    currentKeybind.value = event.payload as string;
  });

  onUnmounted(() => {
    unlisten();
  });
});
</script>

<style lang="scss">
@import '~/assets/css/keybind.scss';
</style>