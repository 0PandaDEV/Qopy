<template>
  <div class="noise"></div>
</template>

<style scoped lang="scss">
.noise {
  position: absolute;
  overflow: hidden;
  top: 1px;
  left: 1px;
  width: calc(100vw - 2px);
  height: calc(100vh - 2px);
  pointer-events: none;
  user-select: none;
  background-image: url("/noise.webp");
  background-repeat: repeat;
  image-rendering: pixelated;
  overflow: hidden;
  mix-blend-mode: multiply;
  opacity: .5;
  border-radius: 12px;
}
</style>
