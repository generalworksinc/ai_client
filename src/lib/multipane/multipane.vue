<template>
  <div :class="classnames" :style="{ cursor, userSelect }" @mousedown="onMouseDown">
    <slot></slot>
  </div>
</template>

<script src="./multipane.js"></script>
