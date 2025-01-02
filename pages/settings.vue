<template>
  <div class="bg">
    <NuxtLink to="/" class="back">
      <img src="../public/back_arrow.svg" />
      <p>Back</p>
    </NuxtLink>
    <div class="bottom-bar">
      <div class="left">
        <img alt="" class="logo" src="../public/logo.png" width="18px" />
        <p>Qopy</p>
      </div>
      <div class="right">
        <div
          @click="saveKeybind"
          class="actions"
          :class="{ disabled: keybind.length === 0 }">
          <p>Save</p>
          <div>
            <img alt="" src="../public/cmd.svg" v-if="os === 'macos'" />
            <img
              alt=""
              src="../public/ctrl.svg"
              v-if="os === 'linux' || os === 'windows'" />
            <img alt="" src="../public/enter.svg" />
          </div>
        </div>
      </div>
    </div>
    <div
      class="keybind-container"
      :class="{ 'empty-keybind': showEmptyKeybindError }">
      <h2 class="title">Record a new Hotkey</h2>
      <div
        @blur="onBlur"
        @focus="onFocus"
        @keydown="onKeyDown"
        class="keybind-input"
        ref="keybindInput"
        tabindex="0">
        <span class="key" v-if="keybind.length === 0">Click here</span>
        <template v-else>
          <span
            :key="index"
            class="key"
            :class="{ modifier: isModifier(key) }"
            v-for="(key, index) in keybind">
            {{ keyToDisplay(key) }}
          </span>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, reactive, ref } from "vue";
import { platform } from "@tauri-apps/plugin-os";
import { useRouter } from "vue-router";
import { Key } from "wrdu-keyboard/key";

const activeModifiers = reactive<Set<string>>(new Set());
const isKeybindInputFocused = ref(false);
const keybind = ref<string[]>([]);
const keybindInput = ref<HTMLElement | null>(null);
const lastBlurTime = ref(0);
const os = ref("");
const router = useRouter();
const keyboard = useKeyboard();
const showEmptyKeybindError = ref(false);

const keyToDisplayMap: Record<string, string> = {
  " ": "Space",
  Alt: "Alt",
  AltLeft: "Alt L",
  AltRight: "Alt R",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  Control: "Ctrl",
  ControlLeft: "Ctrl L",
  ControlRight: "Ctrl R",
  Enter: "↵",
  Meta: "Meta",
  MetaLeft: "Meta L",
  MetaRight: "Meta R",
  Shift: "⇧",
  ShiftLeft: "⇧ L",
  ShiftRight: "⇧ R",
};

const modifierKeySet = new Set([
  "Alt",
  "AltLeft",
  "AltRight",
  "Control",
  "ControlLeft",
  "ControlRight",
  "Meta",
  "MetaLeft",
  "MetaRight",
  "Shift",
  "ShiftLeft",
  "ShiftRight",
]);

const isModifier = (key: string): boolean => {
  return modifierKeySet.has(key);
};

const keyToDisplay = (key: string): string => {
  return keyToDisplayMap[key] || key;
};

const updateKeybind = () => {
  const modifiers = Array.from(activeModifiers).sort();
  const nonModifiers = keybind.value.filter((key) => !isModifier(key));
  keybind.value = [...modifiers, ...nonModifiers];
};

const onBlur = () => {
  isKeybindInputFocused.value = false;
  lastBlurTime.value = Date.now();
  showEmptyKeybindError.value = false;
};

const onFocus = () => {
  isKeybindInputFocused.value = true;
  activeModifiers.clear();
  keybind.value = [];
  showEmptyKeybindError.value = false;
};

const onKeyDown = (event: KeyboardEvent) => {
  const key = event.code;

  if (key === "Escape") {
    if (keybindInput.value) {
      keybindInput.value.blur();
    }
    return;
  }

  if (isModifier(key)) {
    activeModifiers.add(key);
  } else if (!keybind.value.includes(key)) {
    keybind.value = keybind.value.filter((k) => isModifier(k));
    keybind.value.push(key);
  }

  updateKeybind();
  showEmptyKeybindError.value = false;
};

const saveKeybind = async () => {
  if (keybind.value.length > 0) {
    console.log("Saving keybind", keybind.value);
    await invoke("save_keybind", { keybind: keybind });
  } else {
    showEmptyKeybindError.value = true;
  }
};

os.value = platform();

onMounted(() => {
  keyboard.down([Key.Escape], (event) => {
    console.log("Escape key pressed");
    if (isKeybindInputFocused.value) {
      keybindInput.value?.blur();
    } else {
      router.push("/");
    }
  });

  switch (os.value) {
    case "macos":
      keyboard.down([Key.LeftMeta, Key.Enter], (event) => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });

      keyboard.down([Key.RightMeta, Key.Enter], (event) => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });
      break;

    case "linux" || "windows":
      keyboard.down([Key.LeftControl, Key.Enter], (event) => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });

      keyboard.down([Key.RightControl, Key.Enter], (event) => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });
      break;
  }
});

onUnmounted(() => {
  keyboard.unregisterAll();
});
</script>

<style scoped lang="scss">
@use "~/assets/css/settings.scss";
</style>
