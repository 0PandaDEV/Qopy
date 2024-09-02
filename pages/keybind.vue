<template>
  <div class="bg">
    <div class="back">
      <img @click="router.push('/')" src="../public/back_arrow.svg">
      <p>Back</p>
    </div>
    <div class="bottom-bar">
      <div class="left">
        <img alt="" class="logo" src="../public/logo.png" width="18px">
        <p>Qopy</p>
      </div>
      <div class="right">
        <div @click="saveKeybind" class="actions">
          <p>Save</p>
          <div>
            <img alt="" src="../public/ctrl.svg" v-if="os === 'windows' || os === 'linux'">
            <img alt="" src="../public/cmd.svg" v-if="os === 'macos'">
            <img alt="" src="../public/enter.svg">
          </div>
        </div>
      </div>
    </div>
    <div class="keybind-container">
      <h2 class="title">Record a new Hotkey</h2>
      <div @blur="onBlur" @focus="onFocus" @keydown="onKeyDown" @keyup="onKeyUp" class="keybind-input"
        ref="keybindInput" tabindex="0">
        <span class="key" v-if="currentKeybind.length === 0">Click here</span>
        <template v-else>
          <span :key="index" class="key" v-for="(key, index) in currentKeybind">{{ keyToDisplay(key) }}</span>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';
import { onMounted, onUnmounted, ref } from 'vue';
import { useRouter } from 'vue-router';

const activeModifiers = ref<Set<string>>(new Set());
const currentKeybind = ref<string[]>([]);
const isKeybindInputFocused = ref(false);
const keybindInput = ref<HTMLElement | null>(null);
const lastNonModifier = ref('');
const os = ref('');
const router = useRouter();
const lastBlurTime = ref(0);

const keyToDisplayMap: Record<string, string> = {
  " ": "Space",
  Alt: "Alt",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  Control: "Ctrl",
  Enter: "↵",
  Meta: "Meta",
  Shift: "⇧",
};

const modifierKeySet = new Set(["Alt", "Control", "Meta", "Shift"]);

function keyToDisplay(key: string): string {
  return keyToDisplayMap[key] || key.toUpperCase();
}

function updateCurrentKeybind() {
  const modifiers = Array.from(activeModifiers.value);
  currentKeybind.value = lastNonModifier.value ? [...modifiers, lastNonModifier.value] : modifiers;
}

const onBlur = () => {
  isKeybindInputFocused.value = false;
  lastBlurTime.value = Date.now();
};

const onFocus = () => {
  isKeybindInputFocused.value = true;
  activeModifiers.value.clear();
  lastNonModifier.value = '';
  updateCurrentKeybind();
};

const onKeyDown = (event: KeyboardEvent) => {
  event.preventDefault();
  const key = event.key;

  if (key === "Escape") {
    if (keybindInput.value) {
      keybindInput.value.blur();
    }
    return;
  }

  if (modifierKeySet.has(key)) {
    activeModifiers.value.add(key);
  } else {
    lastNonModifier.value = key;
  }
  updateCurrentKeybind();
};

const onKeyUp = (event: KeyboardEvent) => {
  event.preventDefault();
};

const saveKeybind = async () => {
  console.log("New:", currentKeybind.value);
  console.log("Old: " + new Array(await invoke("get_keybind")));
  await invoke("save_keybind", { keybind: currentKeybind.value})
};

const handleGlobalKeyDown = (event: KeyboardEvent) => {
  const now = Date.now();
  if ((os.value === 'macos' ? event.metaKey : event.ctrlKey) && event.key === 'Enter' && !isKeybindInputFocused.value) {
    event.preventDefault();
    saveKeybind();
  } else if (event.key === 'Escape' && !isKeybindInputFocused.value && now - lastBlurTime.value > 100) {
    event.preventDefault();
    router.push('/');
  }
};

onMounted(() => {
  os.value = platform();
  window.addEventListener('keydown', handleGlobalKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown);
});
</script>

<style scoped lang="scss">
@import '~/assets/css/keybind.scss';
</style>