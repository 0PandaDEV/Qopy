<template>
	<div class="noise"></div>
</template>

<style scoped lang="scss">
	.noise {
  position: absolute;
  overflow: hidden;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  pointer-events: none;
  user-select: none;
  z-index: 0;
  background-image: url("/noise.png");
  background-repeat: repeat;
  image-rendering: pixelated;
  overflow: hidden;
}
</style>
