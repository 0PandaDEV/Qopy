<template>
  <div class="bg">
    <div class="keybind-container">
      <h2>Set New Keybind</h2>
      <div class="keybind-input" tabindex="0" @keydown="onKeyDown" @keyup="onKeyUp" @focus="onFocus" ref="keybindInput">
        {{ currentKeybind || 'Click here, then press your desired key combination' }}
      </div>
      <button @click="saveKeybind" :disabled="!currentKeybind">Save Keybind</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const currentKeybind = ref('');
const keybindInput = ref<HTMLElement | null>(null);
const keys = ref<Set<string>>(new Set());
const recording = ref(false);

const keyToDisplayMap: Record<string, string> = {
  " ": "Space",
  Alt: "Alt",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  Control: "Ctrl",
  Enter: "↵",
  Escape: "Esc",
  Meta: "Meta",
  Shift: "⇧",
};

const modifierKeySet = new Set(["Alt", "Control", "Meta", "Shift"]);

function keyCodeToKey(keyCode: string): string {
  if (keyCode.startsWith("Key")) return keyCode.slice(3);
  if (keyCode.endsWith("Left")) return keyCode.slice(0, -4);
  if (keyCode.startsWith("Digit")) return keyCode.slice(5);
  if (keyCode.endsWith("Right")) return keyCode.slice(0, -5);
  return keyCode;
}

function keyToDisplay(keyCode: string): string {
  const key = keyCodeToKey(keyCode);
  return keyToDisplayMap[key] || key;
}

function keyCombToDisplay(keyComb: Set<string>): string {
  return Array.from(keyComb).map(keyToDisplay).join("+");
}

function mapKeyToTauriKey(key: string): string {
  return key === "Meta" ? "Command" : key;
}

const onKeyDown = (event: KeyboardEvent) => {
  event.preventDefault();
  const key = keyCodeToKey(event.code);

  if (modifierKeySet.has(key) && !keys.value.has(key)) {
    keys.value = new Set(Array.from(keys.value).filter(k => modifierKeySet.has(k)));
  }

  keys.value.add(key);
  updateCurrentKeybind();
};

const onKeyUp = (event: KeyboardEvent) => {
  event.preventDefault();
  const key = keyCodeToKey(event.code);
  if (!modifierKeySet.has(key)) {
    recording.value = false;
    updateCurrentKeybind();
  }
};

const onFocus = () => {
  resetKeybind();
};

const updateCurrentKeybind = () => {
  currentKeybind.value = keyCombToDisplay(keys.value);
};

const resetKeybind = () => {
  keys.value.clear();
  currentKeybind.value = '';
  recording.value = true;
};

const saveKeybind = async () => {
  console.log(await invoke("get_keybind"));
};

const startCapture = () => {
  resetKeybind();
};
</script>

<style lang="scss">
@import '~/assets/css/keybind.scss';
</style>