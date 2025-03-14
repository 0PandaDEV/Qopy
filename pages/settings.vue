<template>
  <div class="bg">
    <div class="top-bar">
      <NuxtLink to="/" class="back">
        <img src="../public/back_arrow.svg" />
        <p>Back</p>
      </NuxtLink>
    </div>
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
            <IconsCmd v-if="os === 'macos'" />
            <IconsCtrl v-if="os === 'linux' || os === 'windows'" />
            <IconsEnter />
          </div>
        </div>
      </div>
    </div>
    <div class="settings-container">
      <div class="settings">
        <div class="names">
          <p style="line-height: 14px">Startup</p>
          <p style="line-height: 34px">Qopy Hotkey</p>
        </div>
        <div class="actions">
          <div class="launch">
            <input
              type="checkbox"
              id="launch"
              v-model="autostart"
              @change="toggleAutostart" />
            <label for="launch" class="checkmark">
              <svg
                width="14"
                height="14"
                viewBox="0 0 14 14"
                fill="none"
                xmlns="http://www.w3.org/2000/svg">
                <g>
                  <rect width="14" height="14" />
                  <path
                    id="Path"
                    d="M0 2.00696L2.25015 4.25L6 0"
                    fill="none"
                    stroke-width="1.5"
                    stroke="#E5DFD5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    transform="translate(4 5)" />
                </g>
              </svg>
            </label>
            <p for="launch">Launch Qopy at login</p>
          </div>
          <div
            @blur="onBlur"
            @focus="onFocus"
            class="keybind-input"
            ref="keybindInput"
            tabindex="0"
            :class="{ 'empty-keybind': showEmptyKeybindError }">
            <span class="key" v-if="keybind.length === 0">Click here</span>
            <template v-else>
              <span
                :key="index"
                class="key"
                :class="{ modifier: isModifier(key) }"
                v-for="(key, index) in keybind">
                {{ keyToLabel(key) }}
              </span>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, reactive, ref } from "vue";
import { platform } from "@tauri-apps/plugin-os";
import { useRouter } from "vue-router";
import { KeyValues, KeyLabels } from "../types/keys";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import { Key, keyboard } from "wrdu-keyboard";

const activeModifiers = reactive<Set<KeyValues>>(new Set());
const isKeybindInputFocused = ref(false);
const keybind = ref<KeyValues[]>([]);
const keybindInput = ref<HTMLElement | null>(null);
const lastBlurTime = ref(0);
const os = ref("");
const router = useRouter();
const showEmptyKeybindError = ref(false);
const autostart = ref(false);
const { $settings } = useNuxtApp();

const modifierKeySet = new Set([
  KeyValues.AltLeft,
  KeyValues.AltRight,
  KeyValues.ControlLeft,
  KeyValues.ControlRight,
  KeyValues.MetaLeft,
  KeyValues.MetaRight,
  KeyValues.ShiftLeft,
  KeyValues.ShiftRight,
]);

const isModifier = (key: KeyValues): boolean => {
  return modifierKeySet.has(key);
};

const keyToLabel = (key: KeyValues): string => {
  return KeyLabels[key] || key;
};

const updateKeybind = () => {
  const modifiers = Array.from(activeModifiers);
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
  const key = event.code as KeyValues;

  if (key === KeyValues.Escape) {
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
    await $settings.saveSetting("keybind", JSON.stringify(keybind.value));
    router.push("/");
  } else {
    showEmptyKeybindError.value = true;
  }
};

const toggleAutostart = async () => {
  if (autostart.value === true) {
    await enable();
  } else {
    await disable();
  }
  await $settings.saveSetting("autostart", autostart.value ? "true" : "false");
};

os.value = platform();

onMounted(async () => {
  keyboard.prevent.down([Key.All], (event: KeyboardEvent) => {
    if (isKeybindInputFocused.value) {
      onKeyDown(event);
    }
  });

  keyboard.prevent.down([Key.Escape], () => {
    if (isKeybindInputFocused.value) {
      keybindInput.value?.blur();
    } else {
      router.push("/");
    }
  });

  switch (os.value) {
    case "macos":
      keyboard.prevent.down([Key.LeftMeta, Key.Enter], () => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });

      keyboard.prevent.down([Key.RightMeta, Key.Enter], () => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });
      break;

    case "linux":
    case "windows":
      keyboard.prevent.down([Key.LeftControl, Key.Enter], () => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });

      keyboard.prevent.down([Key.RightControl, Key.Enter], () => {
        if (!isKeybindInputFocused.value) {
          saveKeybind();
        }
      });
      break;
  }

  autostart.value = (await $settings.getSetting("autostart")) === "true";
});

onUnmounted(() => {
  keyboard.clear();
});
</script>

<style scoped lang="scss">
@use "~/assets/css/settings.scss";
</style>
