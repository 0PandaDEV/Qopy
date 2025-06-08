<template>
  <main>
    <div class="top-bar">
      <NuxtLink to="/" class="back">
        <img src="../public/back_arrow.svg" />
        <p>Back</p>
      </NuxtLink>
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
    <BottomBar
      :primary-action="{
        text: 'Save',
        icon: IconsEnter,
        onClick: saveKeybind,
        showModifier: true,
      }" />
  </main>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { platform } from "@tauri-apps/plugin-os";
import { useRouter } from "vue-router";
import { KeyValues, KeyLabels } from "../types/keys";
import { disable, enable } from "@tauri-apps/plugin-autostart";
import { useNuxtApp } from "#app";
import BottomBar from "../components/BottomBar.vue";
import IconsEnter from "~/components/Keys/Enter.vue";

const activeModifiers = reactive<Set<KeyValues>>(new Set());
const isKeybindInputFocused = ref(false);
const keybind = ref<KeyValues[]>([]);
const keybindInput = ref<HTMLElement | null>(null);
const blurredByEscape = ref(false);
const router = useRouter();
const showEmptyKeybindError = ref(false);
const autostart = ref(false);
const { $settings, $keyboard } = useNuxtApp();

const listeners: Array<() => void> = [];

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

const updateKeybindDisplay = () => {
  const modifiers = Array.from(activeModifiers);
  const nonModifiers = keybind.value.filter((key) => !isModifier(key));
  const sortedModifiers = modifiers.sort();
  keybind.value = [...sortedModifiers, ...nonModifiers];
};

const onBlur = () => {
  isKeybindInputFocused.value = false;
  showEmptyKeybindError.value = false;
};

const onFocus = () => {
  isKeybindInputFocused.value = true;
  blurredByEscape.value = false;
  activeModifiers.clear();
  keybind.value = [];
  showEmptyKeybindError.value = false;

  const unlistenAll = $keyboard.listen([$keyboard.Key.All], (event: KeyboardEvent) => {
    event.preventDefault();
    event.stopPropagation();
    const key = event.code as KeyValues;

    if (key === KeyValues.Escape) {
      blurredByEscape.value = true;
      keybindInput.value?.blur();
      return;
    }

    if (isModifier(key)) {
      activeModifiers.add(key);
    } else {
      const nonModifierKey = keybind.value.find(k => !isModifier(k));
      if (!nonModifierKey || nonModifierKey === key) {
         keybind.value = Array.from(activeModifiers);
         if (nonModifierKey !== key) keybind.value.push(key);
      } else {
        keybind.value = [ ...Array.from(activeModifiers), key];
      }
    }
    updateKeybindDisplay();
    showEmptyKeybindError.value = false;
  }, { prevent: true });
  listeners.push(unlistenAll);
};

const saveKeybind = async () => {
  const finalKeybind = keybind.value.filter(k => k);
  if (finalKeybind.length > 0) {
    await $settings.saveSetting("keybind", JSON.stringify(finalKeybind));
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

onMounted(async () => {
  autostart.value = (await $settings.getSetting("autostart")) === "true";

  const metaOrCtrlKey = $keyboard.currentOS === "macos" ? $keyboard.Key.Meta : $keyboard.Key.Control;
  listeners.push(
    $keyboard.listen([metaOrCtrlKey, $keyboard.Key.Enter], saveKeybind, { prevent: true, ignoreIfEditable: true })
  );

  listeners.push(
    $keyboard.listen([$keyboard.Key.Escape], () => {
      if (!isKeybindInputFocused.value && !blurredByEscape.value) {
        router.push("/");
      }
      if(blurredByEscape.value) blurredByEscape.value = false; 
    }, { prevent: true })
  );
});

onUnmounted(() => {
  listeners.forEach(unlisten => unlisten());
  listeners.length = 0;
});
</script>

<style scoped lang="scss">
@use "/styles/settings.scss";
</style>
