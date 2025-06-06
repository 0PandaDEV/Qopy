<template>
  <div class="bottombar">
    <div class="branding">
      <img src="/logo.png" alt="logo" class="logo" />
      <p class="name">Qopy</p>
    </div>
    <div class="buttons">
      <div v-if="primaryAction" class="paste" @click="primaryAction.onClick">
        <p class="text">{{ primaryAction.text }}</p>
        <component :is="primaryAction.icon" />
      </div>
      <div v-if="secondaryAction" class="divider"></div>
      <div v-if="secondaryAction" class="actions" @click="secondaryAction.onClick">
        <p class="text">{{ secondaryAction.text }}</p>
        <div>
          <IconsCtrl v-if="(os === 'windows' || os === 'linux') && secondaryAction.showModifier" />
          <IconsCmd v-if="os === 'macos' && secondaryAction.showModifier" />
          <component :is="secondaryAction.icon" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { platform } from "@tauri-apps/plugin-os";

interface Action {
  text: string;
  icon: any;
  onClick?: () => void;
  showModifier?: boolean;
}

const props = defineProps<{
  primaryAction?: Action;
  secondaryAction?: Action;
}>();

const os = ref<string>("");

onMounted(() => {
  os.value = platform();
});
</script>

<style scoped lang="scss">
.bottombar {
  min-height: 40px;
  width: 100%;
  border-top: 1px solid var(--border);
  backdrop-filter: blur(18px);
  border-radius: 0 0 11px 11px;
  background-color: rgba(46, 45, 43, 0.051);
  display: flex;
  align-items: center;
  padding-left: 11px;
  padding-right: 6px;
  justify-content: space-between;

  .branding {
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--text-secondary);

    .logo {
      width: 18px;
      height: 18px;
    }
  }

  .buttons {
    display: flex;
    align-items: center;

    .text {
      color: var(--text);
    }

    .actions div {
      display: flex;
      align-items: center;
      gap: 2px;
    }

    .divider {
      width: 2px;
      height: 12px;
      background-color: var(--border);
      margin-left: 8px;
      margin-right: 4px;
      transition: all 0.2s;
    }

    .paste,
    .actions {
      padding: 4px;
      padding-left: 8px;
      display: flex;
      align-items: center;
      gap: 8px;
      border-radius: 7px;
      background-color: transparent;
      transition: all 0.2s;
      cursor: pointer;
    }

    .paste:hover,
    .actions:hover {
      background-color: var(--border);
    }

    &:hover .paste:hover ~ .divider,
    &:hover .divider:has(+ .actions:hover) {
      opacity: 0;
    }
  }
}

p {
  font-family: SFRoundedMedium;
}
</style>
