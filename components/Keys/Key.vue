<template>
  <div
    class="key-container"
    :style="{
      backgroundColor: 'var(--border)',
      padding: '0 7px',
      height: '20px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      borderRadius: '4px',
      minWidth: '22px'
    }"
  >
    <span
      :style="{
        color: '#E5E0D5',
        fontSize: '12px'
      }"
    >
      {{ input }}
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  input: string;
}>();
</script>
