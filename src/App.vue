<script setup>

import { emit, listen } from '@tauri-apps/api/event';
// import { register, unregister, unregisterAll  } from '@tauri-apps/api/globalShortcut';
import { useRouter } from 'vue-router';
import { ref, nextTick } from "vue";
import { onMounted, onUnmounted } from '@vue/runtime-core';


const router = useRouter();

onMounted(async () => {
  // await unregister('CommandOrControl+Enter');
  // await unregisterAll();
  // await register('CommandOrControl+Enter', () => {
  //   console.log('Shortcut triggered');
  // });

  //emits
  const unlisten_open_settings = await listen('open_settings', (event) => {
    router.push("/settings");
  });
  const unlisten_open_main = await listen('open_main', (event) => {
    router.push("/");
  });
});
</script>

<template>
  <router-view></router-view>
</template>

<style></style>